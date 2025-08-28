# Claude Diary Hook 📚

A high-performance Rust-based hook for Claude Code that automatically logs your development activities to a SQLite database with intelligent accomplishment inference.

## Features

- 🚀 **Fast**: Written in Rust for minimal overhead
- 🧠 **Smart**: Automatically infers accomplishments from user prompts using pattern matching
- 📊 **Comprehensive**: Tracks tool usage, file modifications, accomplishments, and issues
- 💾 **SQLite Storage**: Reliable database storage with concurrent access support
- ⚡ **Real-time**: Updates database as you work, perfect for multiple Claude Code instances
- 🔍 **Queryable**: View recent entries with built-in commands or via MCP server
- 🛠️ **Configurable**: Customizable diary directory and output format

## Installation

### Prerequisites
- Rust (install from https://rustup.rs/)
- Claude Code CLI

### Quick Setup

1. **Build the hook**:
   ```bash
   cd ~/.claude/hooks/claude-diary-hook
   cargo build --release
   ```

2. **Make it executable**:
   ```bash
   chmod +x target/release/claude-diary-hook
   ```

3. **Create the diary directory**:
   ```bash
   mkdir -p ~/.claude/diaries
   ```

4. **Test the hook**:
   ```bash
   echo '{"event_type": "test", "user_prompt": "Hello World"}' | ./target/release/claude-diary-hook --test --verbose
   ```

5. **Configure Claude Code**:
   ```bash
   # The hook is automatically configured via ~/.claude/settings.json
   # Verify the configuration exists:
   cat ~/.claude/settings.json
   ```
   
   Your settings should contain:
   ```json
   {
     "hooks": {
       "user-prompt-submit": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
     }
   }
   ```

## Usage

### Command Line Options

```bash
claude-diary-hook [OPTIONS]

Options:
  --diary-dir <DIR>    Directory to store diary database [default: ~/.claude/diaries]
  --verbose            Enable verbose output for debugging
  --test              Test mode - prints to stdout instead of writing to database
  --show-recent       Show recent diary entries from database
  --limit <N>         Number of recent sessions to show [default: 5]
  -h, --help          Print help
```

### Examples

**Basic usage** (reads from stdin):
```bash
./claude-diary-hook
```

**View recent diary entries**:
```bash
./claude-diary-hook --show-recent --limit 10
```

**Test mode with custom directory**:
```bash
./claude-diary-hook --test --diary-dir ./my-diaries --verbose
```

**Process a sample event**:
```bash
cat << EOF | ./claude-diary-hook --test
{
  "event_type": "tool_call",
  "timestamp": "2025-08-28T18:30:00Z",
  "tool_calls": [
    {
      "tool_name": "Edit",
      "parameters": {"file_path": "/path/to/file.rs"},
      "duration_ms": 1500,
      "success": true
    }
  ],
  "duration_ms": 2000
}
EOF
```

## Database Storage

The hook stores all data in a SQLite database at:

```
~/.claude/diaries/diary.db
```

### Database Schema

The database contains these tables:

- **sessions**: Main session records with start/end times and durations
- **accomplishments**: What was accomplished (inferred from user prompts)
- **objectives**: Session goals extracted from user inputs
- **issues**: Problems and errors encountered
- **tool_usage**: Claude Code tools used with usage counts
- **files_modified**: Files that were modified during sessions
- **accomplishment_files**: File associations with specific accomplishments

### Key Benefits

- **Concurrent Access**: Multiple Claude Code instances can write simultaneously
- **Real-time Updates**: Data saved immediately as events occur
- **Structured Queries**: Easy to query and analyze your development patterns
- **No File Conflicts**: SQLite handles locking and concurrent writes automatically

### Viewing Diary Entries

**Command Line**:
```bash
# Show recent sessions
./claude-diary-hook --show-recent --limit 5
```

**Sample Output**:
```markdown
=== RECENT DIARY ENTRIES ===

## Session 2025-08-28 13:49:25 - < 1 minute

### ✅ **Accomplishments**

#### **Code Development**
- **Fixed code issues: Fix the authentication bug in auth.rs** _(2000ms)_
  - Files: auth.rs

#### **Documentation**
- **Created documentation: Update API documentation** _(1200ms)_

### 🎯 **Session Objectives**
- Fix the authentication bug in auth.rs
- Update API documentation

### 🛠 **Tools Used**
- Edit: 3 times
- Read: 2 times
- Bash: 1 times

---
```

## Claude Code Integration

### Hook Types

The hook can be configured for different Claude Code events:

```bash
# Check current configuration
cat ~/.claude/settings.json

# The configuration should include:
{
  "hooks": {
    "user-prompt-submit": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
  }
}
```

### Hook Event Types

The hook processes these Claude Code events:
- **user-prompt-submit**: Primary event that captures user inputs and infers accomplishments
- **tool-call**: Secondary events that track tool usage and file modifications
- **session-end**: Finalizes session data


## Smart Accomplishment Inference

The hook automatically categorizes your work based on user prompt patterns:

### Accomplishment Categories

- **Code Development**: "fix", "implement", "create", "refactor", "optimize"
- **Documentation**: "document", "write docs", "readme", "comment"
- **Analysis**: "analyze", "investigate", "research", "study", "examine"
- **Database Operations**: "database", "sql", "schema", "migration"
- **Frontend Development**: "ui", "react", "component", "styling"
- **System Operations**: "configure", "setup", "install", "deploy"
- **Project Management**: "plan", "organize", "todo", "milestone"

### Tool Categories

- **Code Development**: Edit, Write, MultiEdit
- **Code Analysis**: Read, Glob, LS  
- **System Operations**: Bash
- **Code Search**: Grep
- **AI Collaboration**: Task
- **Project Management**: TodoWrite
- **Research**: WebFetch

### Storage Structure

```
~/.claude/diaries/
└── diary.db (SQLite database)
```

## MCP Server Integration

A companion MCP server provides tools to query your diary data:

### Installation

```bash
# Install the MCP server
cd ~/.claude/mcp-servers/diary-server
npm install
npm run build

# Configure Claude Code to use the MCP server
./configure-claude.sh
```

### Usage

Start Claude Code with the MCP server:
```bash
claude --mcp-config ~/.claude/mcp/diary-server.json
```

**Available MCP Tools**:
- `get_today_diary` - Get today's diary entries
- `get_yesterday_diary` - Get yesterday's diary entries  
- `get_diary_by_date` - Get entries for specific date (YYYY-MM-DD)
- `get_recent_sessions` - Get recent diary sessions

**Example Usage**:
```
"Use the get_today_diary tool to show what I worked on today"
"Use the get_recent_sessions tool to show my last 5 sessions"
```

## Troubleshooting

### Common Issues

1. **Hook not triggering**
   ```bash
   # Check if hook is configured
   cat ~/.claude/settings.json
   
   # Verify hook binary exists and is executable
   ls -la ~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook
   ```

2. **Database directory not found**
   ```bash
   mkdir -p ~/.claude/diaries
   ```

3. **Permission denied**
   ```bash
   chmod +x target/release/claude-diary-hook
   ```

4. **Multiple Claude Code instances**
   - SQLite handles concurrent access automatically
   - Each instance creates its own session in the database
   - No conflicts or data loss

5. **Database corruption (rare)**
   ```bash
   # Check database integrity
   sqlite3 ~/.claude/diaries/diary.db "PRAGMA integrity_check;"
   
   # Backup and recreate if needed
   cp ~/.claude/diaries/diary.db ~/.claude/diaries/diary.db.backup
   ```

### Debug Mode

Run with verbose output:
```bash
./claude-diary-hook --verbose --test
```

### Viewing Recent Entries

```bash
# Check if hook is working by viewing recent entries
./claude-diary-hook --show-recent --limit 3
```

### Testing Hook Configuration

```bash
# Test that the hook receives and processes events
echo '{"event_type": "user_prompt", "user_prompt": "Test prompt for debugging"}' | ./target/release/claude-diary-hook --verbose

# Check if data was written to database
./target/release/claude-diary-hook --show-recent --limit 1
```

### Logs

The hook outputs errors to stderr:
- JSON parsing issues
- Database connection problems
- SQLite write errors
- Event processing errors

## Development

### Building from Source

```bash
cd ~/.claude/hooks/claude-diary-hook
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

NONE

---

**Happy coding with automatic diary tracking!** 📝✨