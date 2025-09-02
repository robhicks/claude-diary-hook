use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use clap::Parser;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "claude-diary-hook")]
#[command(about = "Claude Code daily diary hook - logs activities automatically")]
struct Args {
    #[arg(long, help = "Directory to store diary files")]
    diary_dir: Option<PathBuf>,
    
    #[arg(long, help = "Verbose output")]
    verbose: bool,
    
    #[arg(long, help = "Test mode - print to stdout instead of writing")]
    test: bool,
    
    #[arg(long, help = "Show recent diary entries from database")]
    show_recent: bool,
    
    #[arg(long, help = "Number of recent sessions to show", default_value = "5")]
    limit: usize,
}

#[derive(Deserialize, Debug)]
struct ClaudeEvent {
    event_type: String,
    timestamp: Option<String>,
    context: Option<serde_json::Value>,
    session_id: Option<String>,
    user_prompt: Option<String>,
    assistant_response: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    duration_ms: Option<u64>,
    error: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ToolCall {
    tool_name: String,
    parameters: Option<serde_json::Value>,
    result: Option<String>,
    duration_ms: Option<u64>,
    success: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiarySession {
    start_time: DateTime<Local>,
    end_time: Option<DateTime<Local>>,
    objectives: Vec<String>,
    accomplishments: Vec<Accomplishment>,
    issues: Vec<String>,
    files_modified: Vec<String>,
    tool_usage: HashMap<String, u32>,
    total_duration_ms: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Accomplishment {
    category: String,
    description: String,
    duration_ms: Option<u64>,
    files_affected: Vec<String>,
}

impl DiarySession {
    fn new() -> Self {
        Self {
            start_time: Local::now(),
            end_time: None,
            objectives: Vec::new(),
            accomplishments: Vec::new(),
            issues: Vec::new(),
            files_modified: Vec::new(),
            tool_usage: HashMap::new(),
            total_duration_ms: 0,
        }
    }
}

struct DiaryManager {
    db_path: PathBuf,
    current_session_id: Option<i64>,
    current_session: DiarySession,
    verbose: bool,
    test_mode: bool,
}

impl DiaryManager {
    fn new(diary_dir: Option<PathBuf>, verbose: bool, test_mode: bool) -> Result<Self> {
        let diary_dir = diary_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".claude")
        });

        if !test_mode {
            std::fs::create_dir_all(&diary_dir)
                .with_context(|| format!("Failed to create diary directory: {:?}", diary_dir))?;
        }

        let db_path = diary_dir.join("diary.db");
        
        // Handle migration from old directory structure
        if !test_mode {
            let old_db_path = diary_dir.join("diaries").join("diary.db");
            if old_db_path.exists() && !db_path.exists() {
                if verbose {
                    eprintln!("📦 Migrating database from old location: {:?} -> {:?}", old_db_path, db_path);
                }
                std::fs::rename(&old_db_path, &db_path)
                    .with_context(|| format!("Failed to migrate database from {:?} to {:?}", old_db_path, db_path))?;
                    
                // Clean up old directory if it's empty
                if let Ok(entries) = std::fs::read_dir(diary_dir.join("diaries")) {
                    if entries.count() == 0 {
                        let _ = std::fs::remove_dir(diary_dir.join("diaries"));
                    }
                }
            }
        }
        
        let mut manager = Self {
            db_path,
            current_session_id: None,
            current_session: DiarySession::new(),
            verbose,
            test_mode,
        };
        
        if !test_mode {
            manager.init_database()?;
        }
        
        Ok(manager)
    }
    
    fn init_database(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)
            .with_context(|| format!("Failed to open database: {:?}", self.db_path))?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                total_duration_ms INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accomplishments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                duration_ms INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions (id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accomplishment_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                accomplishment_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                FOREIGN KEY (accomplishment_id) REFERENCES accomplishments (id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS objectives (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                objective TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions (id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                issue TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions (id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tool_usage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                tool_name TEXT NOT NULL,
                usage_count INTEGER DEFAULT 1,
                FOREIGN KEY (session_id) REFERENCES sessions (id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files_modified (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions (id)
            )",
            [],
        )?;
        
        if self.verbose {
            eprintln!("Database initialized: {:?}", self.db_path);
        }
        
        Ok(())
    }
    
    fn get_or_create_session(&mut self) -> Result<i64> {
        if let Some(session_id) = self.current_session_id {
            return Ok(session_id);
        }
        
        if self.test_mode {
            self.current_session_id = Some(1);
            return Ok(1);
        }
        
        let conn = Connection::open(&self.db_path)?;
        let session_id = conn.query_row(
            "INSERT INTO sessions (start_time) VALUES (?1) RETURNING id",
            params![self.current_session.start_time.to_rfc3339()],
            |row| row.get(0),
        )?;
        
        self.current_session_id = Some(session_id);
        Ok(session_id)
    }

    fn process_event(&mut self, event: ClaudeEvent) -> Result<()> {
        if self.verbose {
            eprintln!("Processing event: {:?}", event.event_type);
        }

        // Update tool usage statistics
        if let Some(tool_calls) = &event.tool_calls {
            for tool_call in tool_calls {
                *self.current_session.tool_usage
                    .entry(tool_call.tool_name.clone())
                    .or_insert(0) += 1;
            }
        }

        // Add duration
        if let Some(duration) = event.duration_ms {
            self.current_session.total_duration_ms += duration;
        }

        // Analyze event for accomplishments and issues
        match event.event_type.as_str() {
            "session_start" | "user_prompt" | "message" => {
                self.infer_objectives_and_accomplishments(&event);
                // Save immediately for concurrent access
                self.save_current_data()?;
            }
            "tool_call" | "tool_result" => {
                self.process_tool_activity(&event)?;
                // Save immediately for concurrent access
                self.save_current_data()?;
            }
            "error" => {
                self.process_error(&event);
                // Save immediately for concurrent access
                self.save_current_data()?;
            }
            "session_end" => {
                self.current_session.end_time = Some(Local::now());
                self.save_session_to_db()?;
            }
            _ => {
                // Generic processing for other event types
                self.process_generic_activity(&event);
                // Save immediately for concurrent access
                self.save_current_data()?;
            }
        }

        Ok(())
    }
    
    fn save_current_data(&mut self) -> Result<()> {
        if self.test_mode {
            return Ok(());
        }
        
        let session_id = self.get_or_create_session()?;
        let conn = Connection::open(&self.db_path)?;
        
        // Update session duration
        conn.execute(
            "UPDATE sessions SET total_duration_ms = ?1 WHERE id = ?2",
            params![self.current_session.total_duration_ms as i64, session_id],
        )?;
        
        // Save new accomplishments (check if already saved)
        for accomplishment in &self.current_session.accomplishments {
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM accomplishments WHERE session_id = ?1 AND description = ?2)",
                params![session_id, &accomplishment.description],
                |row| row.get(0),
            ).unwrap_or(false);
            
            if !exists {
                let acc_id: i64 = conn.query_row(
                    "INSERT INTO accomplishments (session_id, category, description, duration_ms) 
                     VALUES (?1, ?2, ?3, ?4) RETURNING id",
                    params![
                        session_id,
                        &accomplishment.category,
                        &accomplishment.description,
                        accomplishment.duration_ms.map(|d| d as i64)
                    ],
                    |row| row.get(0),
                )?;
                
                // Save files affected by this accomplishment
                for file_path in &accomplishment.files_affected {
                    conn.execute(
                        "INSERT INTO accomplishment_files (accomplishment_id, file_path) VALUES (?1, ?2)",
                        params![acc_id, file_path],
                    )?;
                }
            }
        }
        
        // Save new objectives
        for objective in &self.current_session.objectives {
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM objectives WHERE session_id = ?1 AND objective = ?2)",
                params![session_id, objective],
                |row| row.get(0),
            ).unwrap_or(false);
            
            if !exists {
                conn.execute(
                    "INSERT INTO objectives (session_id, objective) VALUES (?1, ?2)",
                    params![session_id, objective],
                )?;
            }
        }
        
        // Update tool usage (upsert)
        for (tool_name, count) in &self.current_session.tool_usage {
            conn.execute(
                "INSERT OR REPLACE INTO tool_usage (session_id, tool_name, usage_count) 
                 VALUES (?1, ?2, ?3)",
                params![session_id, tool_name, *count as i64],
            )?;
        }
        
        
        Ok(())
    }

    fn infer_objectives_and_accomplishments(&mut self, event: &ClaudeEvent) {
        if let Some(prompt) = &event.user_prompt {
            // Extract objectives from user prompts
            let objective = if prompt.len() > 100 {
                format!("{}", prompt.chars().take(100).collect::<String>())
            } else {
                prompt.clone()
            };
            
            self.current_session.objectives.push(objective);
            
            // Infer accomplishments from user prompts
            self.infer_accomplishments_from_prompt(prompt, event.duration_ms);
        }
    }

    fn infer_accomplishments_from_prompt(&mut self, prompt: &str, duration_ms: Option<u64>) {
        let prompt_lower = prompt.to_lowercase();
        
        // Define patterns for different types of accomplishments
        let patterns = [
            // Code Development
            ("write|create|implement|add|build|develop|code|program", "Code Development", "Implemented new functionality"),
            ("fix|debug|resolve|solve|repair|correct", "Code Development", "Fixed code issues"),
            ("refactor|optimize|improve|enhance|update", "Code Development", "Improved code quality"),
            ("test|unit test|integration test", "Code Development", "Added tests"),
            
            // Documentation
            ("document|write docs|readme|comment|explain", "Documentation", "Created documentation"),
            
            // Analysis & Research
            ("analyze|investigate|research|study|examine|explore|understand", "Analysis", "Analyzed codebase"),
            ("find|search|look for|locate", "Code Search", "Searched for information"),
            ("review|check|verify|validate", "Code Review", "Reviewed code"),
            
            // Configuration & Setup
            ("configure|setup|install|deploy|initialize", "System Operations", "Configured system"),
            ("migrate|upgrade|update dependencies", "System Operations", "Updated dependencies"),
            
            // Database Operations
            ("database|sql|query|schema|migration", "Database Operations", "Worked with database"),
            
            // UI/UX Work
            ("ui|user interface|frontend|styling|css|design", "Frontend Development", "Worked on user interface"),
            ("component|react|angular|vue", "Frontend Development", "Developed UI components"),
            
            // Planning & Organization
            ("plan|organize|structure|architect|design", "Planning", "Planned project structure"),
            ("todo|task|milestone|goal", "Project Management", "Managed tasks"),
        ];
        
        let mut found_accomplishment = false;
        
        for (pattern, category, default_description) in patterns.iter() {
            if pattern.split('|').any(|p| prompt_lower.contains(p)) {
                let description = self.generate_accomplishment_description(prompt, default_description);
                
                let accomplishment = Accomplishment {
                    category: category.to_string(),
                    description,
                    duration_ms,
                    files_affected: self.extract_files_from_prompt(prompt),
                };
                
                self.current_session.accomplishments.push(accomplishment);
                found_accomplishment = true;
                break; // Only create one accomplishment per prompt to avoid duplicates
            }
        }
        
        // If no specific pattern matched, create a generic accomplishment for non-trivial prompts
        if !found_accomplishment && prompt.len() > 20 {
            let accomplishment = Accomplishment {
                category: "General".to_string(),
                description: self.generate_accomplishment_description(prompt, "Worked on project task"),
                duration_ms,
                files_affected: self.extract_files_from_prompt(prompt),
            };
            
            self.current_session.accomplishments.push(accomplishment);
        }
    }
    
    fn generate_accomplishment_description(&self, prompt: &str, default: &str) -> String {
        // Try to extract a meaningful description from the prompt
        let cleaned_prompt = prompt
            .lines()
            .next() // Take first line
            .unwrap_or(prompt)
            .trim();
            
        if cleaned_prompt.len() > 80 {
            format!("{}: {}", default, &cleaned_prompt[..77].trim())
        } else if cleaned_prompt.len() > 10 {
            format!("{}: {}", default, cleaned_prompt)
        } else {
            default.to_string()
        }
    }
    
    fn extract_files_from_prompt(&self, prompt: &str) -> Vec<String> {
        let mut files = Vec::new();
        
        // Look for common file patterns in the prompt
        let file_patterns = [
            r"[\w/.-]+\.rs",      // Rust files
            r"[\w/.-]+\.js",      // JavaScript files
            r"[\w/.-]+\.ts",      // TypeScript files
            r"[\w/.-]+\.py",      // Python files
            r"[\w/.-]+\.go",      // Go files
            r"[\w/.-]+\.java",    // Java files
            r"[\w/.-]+\.cpp",     // C++ files
            r"[\w/.-]+\.c",       // C files
            r"[\w/.-]+\.h",       // Header files
            r"[\w/.-]+\.json",    // JSON files
            r"[\w/.-]+\.yaml",    // YAML files
            r"[\w/.-]+\.yml",     // YAML files
            r"[\w/.-]+\.toml",    // TOML files
            r"[\w/.-]+\.md",      // Markdown files
        ];
        
        for pattern in &file_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                for mat in regex.find_iter(prompt) {
                    files.push(mat.as_str().to_string());
                }
            }
        }
        
        files.sort();
        files.dedup();
        files
    }

    fn process_tool_activity(&mut self, event: &ClaudeEvent) -> Result<()> {
        if let Some(tool_calls) = &event.tool_calls {
            for tool_call in tool_calls {
                let category = self.categorize_tool(&tool_call.tool_name);
                
                let mut description = format!("Used {} tool", tool_call.tool_name);
                let mut files_affected = Vec::new();
                
                // Extract file information from tool parameters
                if let Some(params) = &tool_call.parameters {
                    if let Some(file_path) = params.get("file_path") {
                        if let Some(path_str) = file_path.as_str() {
                            files_affected.push(path_str.to_string());
                            self.current_session.files_modified.push(path_str.to_string());
                            description = format!("Modified {}", path_str);
                        }
                    }
                }

                let accomplishment = Accomplishment {
                    category,
                    description,
                    duration_ms: tool_call.duration_ms,
                    files_affected,
                };

                self.current_session.accomplishments.push(accomplishment);
            }
        }
        Ok(())
    }

    fn process_error(&mut self, event: &ClaudeEvent) {
        if let Some(error_msg) = &event.error {
            let issue = format!("Error encountered: {}", 
                if error_msg.len() > 150 {
                    format!("{}...", error_msg.chars().take(150).collect::<String>())
                } else {
                    error_msg.clone()
                }
            );
            self.current_session.issues.push(issue);
        }
    }

    fn process_generic_activity(&mut self, event: &ClaudeEvent) {
        // Process other types of activities
        if let Some(response) = &event.assistant_response {
            if response.len() > 50 {
                let activity = format!("Analysis and response provided");
                let accomplishment = Accomplishment {
                    category: "Analysis".to_string(),
                    description: activity,
                    duration_ms: event.duration_ms,
                    files_affected: Vec::new(),
                };
                self.current_session.accomplishments.push(accomplishment);
            }
        }
    }

    fn categorize_tool(&self, tool_name: &str) -> String {
        match tool_name {
            "Edit" | "Write" | "MultiEdit" => "Code Development".to_string(),
            "Read" | "Glob" | "LS" => "Code Analysis".to_string(),
            "Bash" => "System Operations".to_string(),
            "Grep" => "Code Search".to_string(),
            "Task" => "AI Collaboration".to_string(),
            "TodoWrite" => "Project Management".to_string(),
            "WebFetch" => "Research".to_string(),
            _ => "Other".to_string(),
        }
    }

    fn save_session_to_db(&mut self) -> Result<()> {
        if self.test_mode {
            let content = self.generate_diary_content();
            let today = Local::now().format("%Y-%m-%d").to_string();
            println!("=== DIARY ENTRY FOR {} ===", today);
            println!("{}", content);
            return Ok(());
        }
        
        let session_id = self.get_or_create_session()?;
        let conn = Connection::open(&self.db_path)?;
        
        // Update session end time
        conn.execute(
            "UPDATE sessions SET end_time = ?1, total_duration_ms = ?2 WHERE id = ?3",
            params![
                self.current_session.end_time.map(|t| t.to_rfc3339()),
                self.current_session.total_duration_ms as i64,
                session_id
            ],
        )?;
        
        // Save accomplishments
        for accomplishment in &self.current_session.accomplishments {
            let acc_id: i64 = conn.query_row(
                "INSERT INTO accomplishments (session_id, category, description, duration_ms) 
                 VALUES (?1, ?2, ?3, ?4) RETURNING id",
                params![
                    session_id,
                    &accomplishment.category,
                    &accomplishment.description,
                    accomplishment.duration_ms.map(|d| d as i64)
                ],
                |row| row.get(0),
            )?;
            
            // Save files affected by this accomplishment
            for file_path in &accomplishment.files_affected {
                conn.execute(
                    "INSERT INTO accomplishment_files (accomplishment_id, file_path) VALUES (?1, ?2)",
                    params![acc_id, file_path],
                )?;
            }
        }
        
        // Save objectives
        for objective in &self.current_session.objectives {
            conn.execute(
                "INSERT INTO objectives (session_id, objective) VALUES (?1, ?2)",
                params![session_id, objective],
            )?;
        }
        
        // Save issues
        for issue in &self.current_session.issues {
            conn.execute(
                "INSERT INTO issues (session_id, issue) VALUES (?1, ?2)",
                params![session_id, issue],
            )?;
        }
        
        // Save tool usage
        for (tool_name, count) in &self.current_session.tool_usage {
            conn.execute(
                "INSERT INTO tool_usage (session_id, tool_name, usage_count) VALUES (?1, ?2, ?3)",
                params![session_id, tool_name, *count as i64],
            )?;
        }
        
        // Save modified files
        let mut unique_files: Vec<_> = self.current_session.files_modified.iter().collect();
        unique_files.sort();
        unique_files.dedup();
        for file_path in unique_files {
            conn.execute(
                "INSERT INTO files_modified (session_id, file_path) VALUES (?1, ?2)",
                params![session_id, file_path],
            )?;
        }
        
        
        if self.verbose {
            eprintln!("Saved session {} to database: {:?}", session_id, self.db_path);
        }
        
        Ok(())
    }

    fn generate_diary_content(&self) -> String {
        let mut content = String::new();
        
        let duration_mins = self.current_session.total_duration_ms / 60000;
        let duration_display = if duration_mins > 0 {
            format!("~{} minutes", duration_mins)
        } else {
            "< 1 minute".to_string()
        };

        // Group accomplishments by category
        let mut categories: HashMap<String, Vec<&Accomplishment>> = HashMap::new();
        for acc in &self.current_session.accomplishments {
            categories.entry(acc.category.clone()).or_insert(Vec::new()).push(acc);
        }

        content.push_str(&format!("\n### ✅ **Accomplishments** _({})*\n\n", duration_display));

        for (category, accomplishments) in categories {
            content.push_str(&format!("#### **{}**\n", category));
            for acc in accomplishments {
                let duration_str = if let Some(duration) = acc.duration_ms {
                    format!(" _({}ms)_", duration)
                } else {
                    String::new()
                };
                content.push_str(&format!("- **{}**{}\n", acc.description, duration_str));
                
                if !acc.files_affected.is_empty() {
                    content.push_str("  - Files: ");
                    content.push_str(&acc.files_affected.join(", "));
                    content.push_str("\n");
                }
            }
            content.push_str("\n");
        }

        if !self.current_session.objectives.is_empty() {
            content.push_str("### 🎯 **Session Objectives**\n");
            for obj in &self.current_session.objectives {
                content.push_str(&format!("- {}\n", obj));
            }
            content.push_str("\n");
        }

        if !self.current_session.issues.is_empty() {
            content.push_str("### ⚠️ **Issues Encountered**\n");
            for issue in &self.current_session.issues {
                content.push_str(&format!("- {}\n", issue));
            }
            content.push_str("\n");
        }

        if !self.current_session.tool_usage.is_empty() {
            content.push_str("### 🛠 **Tools Used**\n");
            for (tool, count) in &self.current_session.tool_usage {
                content.push_str(&format!("- {}: {} times\n", tool, count));
            }
            content.push_str("\n");
        }

        if !self.current_session.files_modified.is_empty() {
            content.push_str("### 📁 **Files Modified**\n");
            let mut unique_files: Vec<_> = self.current_session.files_modified.iter().collect();
            unique_files.sort();
            unique_files.dedup();
            for file in unique_files {
                content.push_str(&format!("- {}\n", file));
            }
            content.push_str("\n");
        }

        content.push_str("---\n");
        
        content
    }
    
    
    fn show_recent_entries(&self, limit: usize) -> Result<()> {
        if self.test_mode {
            println!("Recent entries not available in test mode");
            return Ok(());
        }
        
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, start_time, end_time, total_duration_ms FROM sessions 
             ORDER BY start_time DESC LIMIT ?1"
        )?;
        
        let session_rows = stmt.query_map([limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        
        println!("\n=== RECENT DIARY ENTRIES ===");
        
        for session_result in session_rows {
            let (session_id, start_time, _end_time, total_duration_ms) = session_result?;
            
            let start_dt = DateTime::parse_from_rfc3339(&start_time)?
                .with_timezone(&Local);
            
            let duration_mins = total_duration_ms / 60000;
            let duration_display = if duration_mins > 0 {
                format!("~{} minutes", duration_mins)
            } else {
                "< 1 minute".to_string()
            };
            
            println!("\n## Session {} - {}", 
                start_dt.format("%Y-%m-%d %H:%M:%S"),
                duration_display
            );
            
            // Get accomplishments
            let mut acc_stmt = conn.prepare(
                "SELECT category, description, duration_ms FROM accomplishments 
                 WHERE session_id = ?1 ORDER BY id"
            )?;
            
            let accomplishments = acc_stmt.query_map([session_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            })?;
            
            let mut categories: HashMap<String, Vec<(String, Option<i64>)>> = HashMap::new();
            for acc_result in accomplishments {
                let (category, description, duration_ms) = acc_result?;
                categories.entry(category).or_insert(Vec::new()).push((description, duration_ms));
            }
            
            if !categories.is_empty() {
                println!("\n### ✅ **Accomplishments**");
                for (category, accs) in categories {
                    println!("\n#### **{}**", category);
                    for (desc, duration_ms) in accs {
                        let duration_str = if let Some(duration) = duration_ms {
                            format!(" _({}ms)_", duration)
                        } else {
                            String::new()
                        };
                        println!("- **{}**{}", desc, duration_str);
                    }
                }
            }
            
            // Get objectives
            let mut obj_stmt = conn.prepare(
                "SELECT objective FROM objectives WHERE session_id = ?1 ORDER BY id"
            )?;
            
            let objectives = obj_stmt.query_map([session_id], |row| {
                Ok(row.get::<_, String>(0)?)
            })?;
            
            let obj_list: Result<Vec<String>, _> = objectives.collect();
            let obj_list = obj_list?;
            
            if !obj_list.is_empty() {
                println!("\n### 🎯 **Session Objectives**");
                for obj in obj_list {
                    println!("- {}", obj);
                }
            }
            
            println!("\n---");
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut diary_manager = DiaryManager::new(args.diary_dir, args.verbose, args.test)?;

    // If user wants to show recent entries, do that and exit
    if args.show_recent {
        return diary_manager.show_recent_entries(args.limit);
    }

    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line.context("Failed to read line from stdin")?;
        
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<ClaudeEvent>(&line) {
            Ok(event) => {
                if let Err(e) = diary_manager.process_event(event) {
                    eprintln!("Error processing event: {}", e);
                }
            }
            Err(e) => {
                if args.verbose {
                    eprintln!("Failed to parse JSON: {} (input: {})", e, line);
                }
                // Try to process as a simple text message
                let simple_event = ClaudeEvent {
                    event_type: "message".to_string(),
                    timestamp: Some(Local::now().to_rfc3339()),
                    context: None,
                    session_id: None,
                    user_prompt: Some(line),
                    assistant_response: None,
                    tool_calls: None,
                    duration_ms: None,
                    error: None,
                };
                if let Err(e) = diary_manager.process_event(simple_event) {
                    eprintln!("Error processing simple event: {}", e);
                }
            }
        }
    }

    // Handle session end if not explicitly received  
    diary_manager.current_session.end_time = Some(Local::now());
    diary_manager.save_session_to_db()?;

    Ok(())
}