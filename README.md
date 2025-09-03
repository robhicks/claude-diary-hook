# Claude Diary Hook 📚

[![GitHub](https://img.shields.io/badge/GitHub-robhicks%2Fclaude--diary--hook-blue?logo=github)](https://github.com/robhicks/claude-diary-hook)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-3.x-green?logo=sqlite)](https://www.sqlite.org/)
[![License](https://img.shields.io/badge/License-Public%20Domain-brightgreen)](#license)

A high-performance Rust-based hook for Claude Code that automatically logs your development activities to a SQLite database with intelligent accomplishment inference.

## Quick Links

- 🏠 **Repository**: [github.com/robhicks/claude-diary-hook](https://github.com/robhicks/claude-diary-hook)
- 📥 **Download**: [Latest Release](https://github.com/robhicks/claude-diary-hook/releases)
- 🐛 **Issues**: [Report Bug](https://github.com/robhicks/claude-diary-hook/issues)
- 💡 **Feature Requests**: [Request Feature](https://github.com/robhicks/claude-diary-hook/issues/new)

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

1. **Clone the repository**:
   ```bash
   git clone https://github.com/robhicks/claude-diary-hook.git ~/.claude/hooks/claude-diary-hook
   cd ~/.claude/hooks/claude-diary-hook
   ```

2. **Install and build**:
   ```bash
   ./install.sh
   ```
   This automatically builds the project and creates the diary directory.

3. **Test the installation**:
   ```bash
   ./test.sh
   ```
   This runs comprehensive tests to verify everything works correctly.

### Manual Setup (Alternative)

If you prefer manual installation:

1. **Build the hook**:
   ```bash
   cargo build --release
   ```

2. **Make it executable**:
   ```bash
   chmod +x target/release/claude-diary-hook
   ```

3. **Create the diary directory**:
   ```bash
   mkdir -p ~/.claude
   ```

4. **Test the hook**:
   ```bash
   echo '{"event_type": "test", "user_prompt": "Hello World"}' | ./target/release/claude-diary-hook --test --verbose
   ```

## Configuration

**Configure Claude Code**:
   ```bash
   # Configuration goes in ~/.claude/settings.json
   # Verify the configuration exists:
   cat ~/.claude/settings.json
   ```
   
   **Minimal configuration** (recommended - captures user prompts):
   ```json
   {
     "hooks": {
       "UserpromptSubmit": [
         {
           "matcher": "*",
           "hooks": [
             {
               "type": "command",
               "command": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
             }
           ]
         }
       ]
     }
   }
   ```

   **Full configuration** (captures more events for detailed tracking):
   ```json
   {
     "hooks": {
       "UserpromptSubmit": [
         {
           "matcher": "*",
           "hooks": [
             {
               "type": "command",
               "command": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
             }
           ]
         }
       ],
       "PostToolUse": [
         {
           "matcher": "*",
           "hooks": [
             {
               "type": "command",
               "command": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
             }
           ]
         }
       ],
       "SessionEnd": [
         {
           "matcher": "*",
           "hooks": [
             {
               "type": "command",
               "command": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
             }
           ]
         }
       ]
     }
   }
   ```

## Usage

### Command Line Options

```bash
claude-diary-hook [OPTIONS]

Options:
  --diary-dir <DIR>    Directory to store diary database [default: ~/.claude]
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
~/.claude/diary.db
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
- **Smart JSON Parsing**: Automatically extracts meaningful prompts from nested JSON event data

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
```

### Hook Event Types

The hook processes these Claude Code events:
- **UserpromptSubmit**: Primary event that captures user inputs and infers accomplishments (required)
- **PostToolUse**: Tracks tool usage and file modifications after each tool call (optional, provides more detail)
- **SessionEnd**: Finalizes session data when a session ends (optional, helps with session completion tracking)

**Note**: The minimal configuration with just `UserpromptSubmit` is sufficient for most users and captures all essential diary information. Event names are case-sensitive.


## Smart Accomplishment Inference

The hook automatically categorizes your work based on user prompt patterns. It intelligently handles both plain text prompts and nested JSON formats (automatically parsing JSON objects to extract the actual prompt text):

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
~/.claude/
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

## Upgrading

### Automatic Migration

When upgrading from previous versions that used `~/.claude/diaries/diary.db`, the hook will automatically:

1. **Detect** the old database location
2. **Move** your existing data to the new location (`~/.claude/diary.db`)
3. **Clean up** the empty old directory
4. **Continue** working with all your historical data intact

**What you need to do**: Nothing! Just rebuild and run as normal:

```bash
./install.sh
```

The migration message will appear in verbose mode:
```bash
./claude-diary-hook --verbose
# 📦 Migrating database from old location: ~/.claude/diaries/diary.db -> ~/.claude/diary.db
```

### Manual Migration (if needed)

If automatic migration fails for any reason:

```bash
# Move the database manually
mv ~/.claude/diaries/diary.db ~/.claude/diary.db

# Remove empty directory
rmdir ~/.claude/diaries
```

**Your data is preserved** - no diary entries will be lost during the upgrade process.

## Troubleshooting

### Common Issues

1. **Diary showing raw JSON instead of meaningful text**
   - This was fixed in the latest version with improved JSON parsing
   - Rebuild the hook: `cargo build --release`
   - The hook now automatically extracts actual prompts from nested JSON data

2. **Hook not triggering**
   ```bash
   # Check if hook is configured
   cat ~/.claude/settings.json
   
   # Verify hook binary exists and is executable
   ls -la ~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook
   ```

3. **Database directory not found**
   ```bash
   mkdir -p ~/.claude
   ```

4. **Permission denied**
   ```bash
   chmod +x target/release/claude-diary-hook
   ```

5. **Multiple Claude Code instances**
   - SQLite handles concurrent access automatically
   - Each instance creates its own session in the database
   - No conflicts or data loss

6. **Database corruption (rare)**
   ```bash
   # Check database integrity
   sqlite3 ~/.claude/diary.db "PRAGMA integrity_check;"
   
   # Backup and recreate if needed
   cp ~/.claude/diary.db ~/.claude/diary.db.backup
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

We welcome contributions! Here's how to get started:

1. **Fork the repository** on [GitHub](https://github.com/robhicks/claude-diary-hook)
2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/claude-diary-hook.git
   cd claude-diary-hook
   ```
3. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes** and add tests
5. **Test your changes**:
   ```bash
   cargo test
   cargo build --release
   ```
6. **Commit and push**:
   ```bash
   git commit -m "Add your feature description"
   git push origin feature/your-feature-name
   ```
7. **Submit a pull request** on GitHub

### Reporting Issues

Found a bug or have a feature request? Please [open an issue](https://github.com/robhicks/claude-diary-hook/issues) on GitHub.

## License

This project is released into the **Public Domain**. You are free to use, modify, and distribute this software without any restrictions.

## Changelog

### v0.1.1 (2025-09-03)
- **Fixed**: JSON parsing issue where raw JSON objects were displayed instead of meaningful accomplishments
- **Added**: Smart JSON parsing to automatically extract actual prompts from nested event data
- **Improved**: Enhanced troubleshooting documentation

### v0.1.0 (2025-08-28)
- Initial release with SQLite storage and accomplishment inference

## Acknowledgments

- Built with ❤️ for the Claude Code community
- Created with assistance from [Claude Code](https://claude.ai/code)

---

**Happy coding with automatic diary tracking!** 📝✨