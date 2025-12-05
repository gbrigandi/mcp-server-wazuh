//! Tests for MCP protocol communication via stdio
//! 
//! These tests verify the basic MCP protocol implementation without
//! requiring a Wazuh connection.

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use tokio::time::sleep;
use serde_json::{json, Value};

struct McpStdioClient {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl McpStdioClient {
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "mcp-server-wazuh"])
            .env("WAZUH_API_HOST", "nonexistent.example.com") // Use non-existent host
            .env("WAZUH_API_PORT", "9999")
            .env("WAZUH_API_USER", "test")
            .env("WAZUH_API_PASS", "test")
            .env("WAZUH_INDEXER_HOST", "nonexistent.example.com") // Use non-existent host
            .env("WAZUH_INDEXER_PORT", "8888")
            .env("WAZUH_INDEXER_USER", "test")
            .env("WAZUH_INDEXER_PASS", "test")
            .env("VERIFY_SSL", "false")
            .env("RUST_LOG", "error") // Minimize logging noise
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Changed from Stdio::null() to inherit stderr
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        Ok(McpStdioClient {
            child,
            stdin,
            stdout,
        })
    }

    fn send_message(&mut self, message: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let message_str = serde_json::to_string(message)?;
        writeln!(self.stdin, "{}", message_str)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        let response: Value = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    fn send_and_receive(&mut self, message: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        self.send_message(message)?;
        self.read_response()
    }
}

impl Drop for McpStdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test]
async fn test_mcp_protocol_initialization() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = McpStdioClient::start()?;
    
    // Give the server time to start
    sleep(Duration::from_millis(500)).await;

    // Test initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let response = client.send_and_receive(&init_request)?;

    // Verify JSON-RPC 2.0 compliance
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["error"].is_null());

    // Verify MCP initialize response structure
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"].is_object());
    assert!(result["serverInfo"].is_object());

    // Verify server info
    let server_info = &result["serverInfo"];
    assert!(server_info["name"].is_string());
    assert!(server_info["version"].is_string());

    Ok(())
}

#[tokio::test]
async fn test_mcp_tools_list_without_wazuh() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = McpStdioClient::start()?;
    
    sleep(Duration::from_millis(500)).await;

    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    });
    client.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client.send_message(&initialized)?;

    // Request tools list
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = client.send_and_receive(&tools_request)?;

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"].is_object());
    
    let result = &response["result"];
    assert!(result["tools"].is_array());
    
    let tools = result["tools"].as_array().unwrap();
    assert!(!tools.is_empty());
    
    // Verify tool structure
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["inputSchema"].is_object());
    }

    Ok(())
}

#[tokio::test]
async fn test_invalid_json_rpc_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = McpStdioClient::start()?;
    
    sleep(Duration::from_millis(500)).await;

    // 1. Initialize the connection first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });
    let _init_response = client.send_and_receive(&init_request)?; // Read and ignore/assert init response
    // assert!(_init_response["result"].is_object()); // Optional: assert successful init

    // 2. Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client.send_message(&initialized_notification)?;

    // 3. Send the invalid JSON-RPC request (missing required fields)
    let invalid_request = json!({
        // "jsonrpc": "2.0", // Missing jsonrpc field to make it invalid
        "id": 2, // Use a new ID
        "method": "some_method_that_might_not_exist"
    });
    client.send_message(&invalid_request)?;

    // 4. The server currently closes the connection upon such an invalid request (see logs:
    //    `ERROR rmcp::transport::io ... serde error ...` followed by `input stream terminated`).
    //    Therefore, subsequent requests should fail. This test verifies this behavior.
    //    Ideally, the server might send a JSON-RPC error and keep the connection open,
    //    but that would require changes to the server's error handling logic.

    // 5. Attempt to send a subsequent valid request.
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 3, // New ID
        "method": "tools/list",
        "params": {}
    });

    let result = client.send_and_receive(&list_tools_request);
    
    // Assert that the operation failed, indicating the connection was likely closed.
    assert!(result.is_err(), "Server should have closed the connection after the invalid request, leading to an error here.");
    
    // Optionally, check the error type more specifically if needed, e.g., for EOF.
    if let Err(e) = result {
        let error_message = e.to_string().to_lowercase();
        assert!(
            error_message.contains("eof") || error_message.contains("broken pipe") || error_message.contains("connection reset"),
            "Expected EOF, broken pipe, or connection reset error, but got: {}", e
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_unsupported_method() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = McpStdioClient::start()?;
    
    sleep(Duration::from_millis(500)).await;

    // Initialize first
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    });
    client.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client.send_message(&initialized_notification)?;

    let unsupported_request = json!({
        "jsonrpc": "2.0",
        "id": 2,

        "method": "unsupported/method"
        // Omitting "params": {} as it might be causing deserialization issues
        // in rmcp for unknown methods. The JSON-RPC spec allows params to be omitted.
    });

    // Send the unsupported request. We don't expect a valid JSON-RPC response.
    // Instead, the server is likely to close the connection due to deserialization issues
    // in rmcp when encountering an unknown method, as it cannot match it to a known JsonRpcMessage variant.
    client.send_message(&unsupported_request)?;

    // Attempt to send a subsequent valid request to confirm the connection was dropped.
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "id": 3, // Use a new ID
        "method": "tools/list",
        "params": {}
    });

    let result = client.send_and_receive(&list_tools_request);
    
    // Assert that the operation failed, indicating the connection was likely closed.
    assert!(result.is_err(), "Server should have closed the connection after the unsupported method request, leading to an error here.");
    
    // Optionally, check the error type more specifically if needed, e.g., for EOF.
    if let Err(e) = result {
        let error_message = e.to_string().to_lowercase();
        assert!(
            error_message.contains("eof") || error_message.contains("broken pipe") || error_message.contains("connection reset"),
            "Expected EOF, broken pipe, or connection reset error, but got: {}", e
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_requests() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = McpStdioClient::start()?;
    
    sleep(Duration::from_millis(500)).await;

    // Initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    });
    client.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client.send_message(&initialized)?;

    // Send multiple requests with different IDs
    let request1 = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/list",
        "params": {}
    });

    let request2 = json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "tools/list",
        "params": {}
    });

    // Send both requests
    client.send_message(&request1)?;
    client.send_message(&request2)?;

    // Read both responses
    let response1 = client.read_response()?;
    let response2 = client.read_response()?;

    // Responses should maintain request IDs (though order might vary)
    let ids: Vec<i64> = vec![
        response1["id"].as_i64().unwrap(),
        response2["id"].as_i64().unwrap(),
    ];
    
    assert!(ids.contains(&10));
    assert!(ids.contains(&20));

    Ok(())
}
