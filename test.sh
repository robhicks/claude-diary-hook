#!/bin/bash

set -e

HOOK_BINARY="./target/release/claude-diary-hook"

if [ ! -f "$HOOK_BINARY" ]; then
    echo "❌ Hook binary not found. Run ./build.sh first"
    exit 1
fi

echo "🧪 Testing Claude Diary Hook..."
echo ""

# Test 1: Basic functionality
echo "📝 Test 1: Basic event processing"
echo '{"event_type": "session_start", "user_prompt": "Help me fix the CloudFormation stack deployment issues"}' | $HOOK_BINARY --test --verbose > /tmp/test1_output.txt

if grep -q "Session Objectives" /tmp/test1_output.txt; then
    echo "✅ Test 1 passed - Basic event processing works"
else
    echo "❌ Test 1 failed - Basic event processing broken"
    cat /tmp/test1_output.txt
fi

# Test 2: Tool call processing  
echo "📝 Test 2: Tool call event processing"
echo '{"event_type": "tool_call", "tool_calls": [{"tool_name": "Edit", "parameters": {"file_path": "/Users/robhicks/dev/contracts-explorer/aws-config/cloudformation/pipeline-stack.yaml"}, "duration_ms": 1500, "success": true}, {"tool_name": "Bash", "parameters": {"command": "aws cloudformation describe-stacks"}, "duration_ms": 800, "success": true}], "duration_ms": 2500}' | $HOOK_BINARY --test > /tmp/test2_output.txt

if grep -q "Code Development" /tmp/test2_output.txt && grep -q "System Operations" /tmp/test2_output.txt; then
    echo "✅ Test 2 passed - Tool call categorization works"
else
    echo "❌ Test 2 failed - Tool call categorization broken"
    cat /tmp/test2_output.txt
fi

# Test 3: Error handling
echo "📝 Test 3: Error event processing"
echo '{"event_type": "error", "error": "Properties validation failed for resource EKSCluster with message: #/ResourcesVpcConfig: extraneous key [EndpointConfig] is not permitted", "duration_ms": 100}' | $HOOK_BINARY --test > /tmp/test3_output.txt

if grep -q "Issues Encountered" /tmp/test3_output.txt; then
    echo "✅ Test 3 passed - Error handling works"
else
    echo "❌ Test 3 failed - Error handling broken" 
    cat /tmp/test3_output.txt
fi

# Test 4: Multiple events in sequence
echo "📝 Test 4: Multiple sequential events"
{
  echo '{"event_type": "session_start", "user_prompt": "Deploy CloudFormation stack"}'
  echo '{"event_type": "tool_call", "tool_calls": [{"tool_name": "Read", "parameters": {"file_path": "pipeline-stack.yaml"}, "success": true}]}'
  echo '{"event_type": "tool_call", "tool_calls": [{"tool_name": "Edit", "parameters": {"file_path": "pipeline-stack.yaml"}, "success": true}]}'
  echo '{"event_type": "session_end"}'
} | $HOOK_BINARY --test > /tmp/test4_output.txt

if grep -q "Code Analysis" /tmp/test4_output.txt && grep -q "Code Development" /tmp/test4_output.txt; then
    echo "✅ Test 4 passed - Sequential events work"
else
    echo "❌ Test 4 failed - Sequential events broken"
    cat /tmp/test4_output.txt
fi

# Test 5: Invalid JSON handling
echo "📝 Test 5: Invalid JSON handling"
echo "invalid json input" | $HOOK_BINARY --test --verbose > /tmp/test5_output.txt 2>&1

if grep -q "message" /tmp/test5_output.txt; then
    echo "✅ Test 5 passed - Invalid JSON handled gracefully"
else
    echo "❌ Test 5 failed - Invalid JSON handling broken"
    cat /tmp/test5_output.txt
fi

# Test 6: Database functionality (creates database only)
echo "📝 Test 6: Database query capability"
TEST_DIR="/tmp/claude-diary-test"
mkdir -p "$TEST_DIR"

echo '{"event_type": "tool_call", "tool_calls": [{"tool_name": "Write", "parameters": {"file_path": "/test/file.txt"}, "success": true}]}' | $HOOK_BINARY --diary-dir "$TEST_DIR" > /dev/null

DB_FILE="$TEST_DIR/diary.db"

if [ -f "$DB_FILE" ]; then
    QUERY_OUTPUT=$($HOOK_BINARY --diary-dir "$TEST_DIR" --show-recent --limit 1)
    if echo "$QUERY_OUTPUT" | grep -q "Code Development"; then
        echo "✅ Test 6 passed - Database query works"
        echo "   📄 Database: $DB_FILE"
        echo "   📝 Query verification passed"
    else
        echo "❌ Test 6 failed - Database query broken"
    fi
else
    echo "❌ Test 6 failed - Database not created"
fi

# Test 7: Database migration from old location
echo "📝 Test 7: Database migration from old directory structure"
MIGRATION_TEST_DIR="/tmp/claude-diary-migration-test"
OLD_DB_DIR="$MIGRATION_TEST_DIR/diaries"
mkdir -p "$OLD_DB_DIR"

# Create a database in the old location
echo '{"event_type": "tool_call", "tool_calls": [{"tool_name": "Edit", "parameters": {"file_path": "/test/migration.txt"}, "success": true}]}' | $HOOK_BINARY --diary-dir "$OLD_DB_DIR" > /dev/null

OLD_DB_FILE="$OLD_DB_DIR/diary.db"
NEW_DB_FILE="$MIGRATION_TEST_DIR/diary.db"

if [ -f "$OLD_DB_FILE" ]; then
    # Test migration by running with the parent directory
    echo '{"event_type": "session_start", "user_prompt": "Test migration"}' | $HOOK_BINARY --diary-dir "$MIGRATION_TEST_DIR" --verbose > /tmp/migration_output.txt 2>&1
    
    if [ -f "$NEW_DB_FILE" ] && [ ! -f "$OLD_DB_FILE" ]; then
        echo "✅ Test 7 passed - Database migration works"
        echo "   📦 Old database moved to new location"
        if grep -q "Migrating database" /tmp/migration_output.txt; then
            echo "   📝 Migration message displayed correctly"
        fi
    else
        echo "❌ Test 7 failed - Database migration broken"
        echo "   Old DB exists: $([ -f "$OLD_DB_FILE" ] && echo "yes" || echo "no")"
        echo "   New DB exists: $([ -f "$NEW_DB_FILE" ] && echo "yes" || echo "no")"
    fi
else
    echo "❌ Test 7 failed - Could not create test database in old location"
fi

# Cleanup
rm -f /tmp/test*_output.txt /tmp/migration_output.txt
rm -rf "$TEST_DIR" "$MIGRATION_TEST_DIR"

echo ""
echo "🎉 All 7 tests completed!"
echo ""
echo "📖 Sample diary output:"
echo "────────────────────────────────────────"
echo '{"event_type": "tool_call", "user_prompt": "Fix EKS configuration in CloudFormation", "tool_calls": [{"tool_name": "Edit", "parameters": {"file_path": "pipeline-stack.yaml"}, "duration_ms": 2000, "success": true}], "duration_ms": 3000}' | $HOOK_BINARY --test
echo "────────────────────────────────────────"
echo ""
echo "Ready for production use! 🚀"