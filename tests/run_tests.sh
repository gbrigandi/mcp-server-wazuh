#!/bin/bash

# Test script for Wazuh MCP Server (rmcp-based)
# This script runs various tests to ensure the server is working correctly

set -e

echo "Starting Wazuh MCP Server tests (rmcp-based)..."

# Set test environment variables
export RUST_LOG=info

echo "Environment variables set:"
echo "  RUST_LOG: $RUST_LOG"

# Function to cleanup background processes
cleanup() {
    echo "Cleaning up..."
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
        wait $SERVER_PID 2>/dev/null || true
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT

echo ""
echo "=== Running Unit Tests ==="
cargo test --lib

echo ""
echo "=== Running MCP Protocol Tests ==="
cargo test --test mcp_stdio_test

echo ""
echo "=== Running Integration Tests with Mock Wazuh ==="
cargo test --test rmcp_integration_test

echo ""
echo "=== Testing Server Binary ==="
echo "Verifying server binary can start and show help..."

# Test that the binary can start and show help. It should exit immediately.
cargo run --bin mcp-server-wazuh -- --help > /dev/null

echo "Help command test completed"

echo ""
echo "=== All Tests Complete ==="
echo ""
echo "Test Summary:"
echo "✓ Unit tests for library components"
echo "✓ Wazuh client tests with mock HTTP server"
echo "✓ MCP protocol tests via stdio"
echo "✓ Integration tests with mock Wazuh API"
echo "✓ Server binary startup test"
echo ""
echo "To test manually with a real Wazuh instance:"
echo "  export WAZUH_HOST=your-wazuh-host"
echo "  export WAZUH_PORT=9200"
echo "  export WAZUH_USER=admin"
echo "  export WAZUH_PASS=your-password"
echo "  cargo run --bin mcp-server-wazuh"
echo ""
echo "Then send MCP commands via stdin, for example:"
echo '  echo '"'"'{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'"'"' | cargo run --bin mcp-server-wazuh'
