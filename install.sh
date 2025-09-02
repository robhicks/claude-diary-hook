#!/bin/bash

set -e

echo "🔧 Installing Claude Diary Hook..."

# Build the project
./build.sh

# Create Claude directory  
CLAUDE_DIR="$HOME/.claude"
mkdir -p "$CLAUDE_DIR"
echo "📁 Created Claude directory: $CLAUDE_DIR"

# Get the absolute path to the binary
HOOK_PATH="$(pwd)/target/release/claude-diary-hook"

echo ""
echo "✅ Installation complete!"
echo ""
echo "🎯 Next steps to integrate with Claude Code:"
echo ""
echo "1. Add this hook to your Claude Code configuration:"
echo ""
echo "   Hook path: $HOOK_PATH"
echo ""
echo "2. Example Claude Code hooks configuration:"
echo '   {
     "hooks": {
       "user-prompt-submit": "'$HOOK_PATH'",
       "tool-call": "'$HOOK_PATH'", 
       "session-end": "'$HOOK_PATH'"
     }
   }'
echo ""
echo "3. Test the installation:"
echo "   ./test.sh"
echo ""
echo "4. Your diary will be automatically created at:"
echo "   $CLAUDE_DIR/diary.db"
echo ""
echo "🎉 Happy automatic diary tracking!"