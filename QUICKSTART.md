# Quick Start Guide 🚀

Get your automatic daily diary up and running in 3 steps!

## 1. Install & Build

```bash
cd ~/.claude/hooks/claude-diary-hook
./install.sh
```

## 2. Test It Works

```bash
./test.sh
```

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

Your diaries will automatically appear in `~/.claude/diaries/` 

**Example diary entry:**
```
~/.claude/diaries/DIARY_2025-08-28.md
```

## Need Help?

- Read `README.md` for full documentation
- Run `./test.sh` to verify everything works
- Check `claude-config-example.json` for advanced configuration

**Happy automatic diary tracking!** 📝