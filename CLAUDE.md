# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Test
```bash
# Build the Rust binary
cargo build --release

# Run tests
cargo test

# Run the installation script (builds and sets up)
./install.sh

# Run comprehensive tests
./test.sh
```

### Hook Testing
```bash
# Test hook with sample event
echo '{"event_type": "test", "user_prompt": "Test prompt"}' | ./target/release/claude-diary-hook --test --verbose

# View recent diary entries
./target/release/claude-diary-hook --show-recent --limit 10
```

## Architecture

This is a Rust-based CLI tool that acts as a Claude Code hook to automatically log development activities to a SQLite database.

### Core Components

- **main.rs**: Entry point containing the DiaryManager that processes Claude Code events
- **Event Processing**: Handles `user-prompt-submit`, `tool-call`, and `session-end` events from Claude Code
- **SQLite Database**: Stores data in `~/.claude/diary.db` with tables for sessions, accomplishments, objectives, issues, tool usage, and file modifications
- **Accomplishment Inference**: Pattern matching on user prompts to categorize work (code development, documentation, analysis, etc.)

### Database Schema

The SQLite database uses these main tables:
- `sessions`: Session records with timing and duration
- `accomplishments`: Inferred from user prompts with categories
- `objectives`: Goals extracted from user inputs  
- `issues`: Errors and problems encountered
- `tool_usage`: Claude Code tools used with counts
- `files_modified`: Files changed during sessions
- `accomplishment_files`: Links accomplishments to affected files

### Key Features

- Automatic migration from old database location (`~/.claude/diaries/diary.db` → `~/.claude/diary.db`)
- Concurrent access support for multiple Claude Code instances
- Pattern-based accomplishment categorization from user prompts
- Smart JSON parsing to extract meaningful prompts from nested event data
- Tool usage tracking and categorization

### Hook Integration

The binary reads JSON events from stdin and processes them based on event type. It's configured in Claude Code's settings.json using the new hook format:
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

Event names are case-sensitive (e.g., `UserpromptSubmit`, `PostToolUse`, `SessionEnd`).