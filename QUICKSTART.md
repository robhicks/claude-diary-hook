# Quick Start Guide 🚀

Get your automatic daily diary up and running in 3 steps!

## 1. Install & Build

```bash
cd ~/.claude/hooks/claude-diary-hook
./install.sh
```

This will build the project and create the diary directory automatically.

## 2. Test It Works

```bash
./test.sh
```

This runs comprehensive tests to verify all functionality works correctly.

## 3. Configure Claude Code

Add the hook to your Claude Code settings:

```json
{
  "hooks": {
    "user-prompt-submit": "~/.claude/hooks/claude-diary-hook/target/release/claude-diary-hook"
  }
}
```

## Done! ✨

Your diaries will automatically be stored in `~/.claude/diary.db` 

**View your diary entries:**
```bash
./target/release/claude-diary-hook --show-recent --limit 5
```

## Need Help?

- Read `README.md` for full documentation
- Run `./test.sh` to verify everything works
- Check `claude-config-example.json` for advanced configuration

**Happy automatic diary tracking!** 📝