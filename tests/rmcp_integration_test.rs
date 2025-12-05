//! Integration tests for the rmcp-based Wazuh MCP Server
//!
//! These tests verify the MCP server functionality using a mock Wazuh API server.
//! Tests cover tool registration, parameter validation, alert retrieval, and error handling.

// Allow holding mutex guard across await in tests - this is intentional for test serialization
#![allow(clippy::await_holding_lock)]

use std::process::{Child, Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use tokio::time::sleep;
use serde_json::{json, Value};
use once_cell::sync::Lazy;
use std::sync::Mutex;

mod mock_wazuh_server;
use mock_wazuh_server::MockWazuhServer;

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

struct McpServerProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl McpServerProcess {
    fn start_with_mock_wazuh(mock_server: &MockWazuhServer) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "mcp-server-wazuh"])
            .env("WAZUH_INDEXER_HOST", mock_server.host())
            .env("WAZUH_INDEXER_PORT", mock_server.port().to_string())
            .env("WAZUH_INDEXER_USER", "admin")
            .env("WAZUH_INDEXER_PASS", "admin")
            .env("VERIFY_SSL", "false")
            .env("WAZUH_TEST_PROTOCOL", "http")
            .env("RUST_LOG", "warn") // Reduce noise in tests
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Inherit stderr to see server logs
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        Ok(McpServerProcess {
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

impl Drop for McpServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test]
async fn test_mcp_server_initialization() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::new();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
    // Give the server time to start
    sleep(Duration::from_millis(500)).await;

    // Send initialize request
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

    let response = mcp_server.send_and_receive(&init_request)?;

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"].is_object());
    assert!(result["serverInfo"].is_object());
    assert!(result["instructions"].is_string());

    Ok(())
}

#[tokio::test]
async fn test_tools_list() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::new();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Request tools list
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = mcp_server.send_and_receive(&tools_request)?;

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"]["tools"].is_array());
    
    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(!tools.is_empty());
    
    // Check for our Wazuh alert summary tool
    let alert_tool = tools.iter()
        .find(|tool| tool["name"] == "get_wazuh_alert_summary")
        .expect("get_wazuh_alert_summary tool should be present");
    
    assert!(alert_tool["description"].is_string());
    assert!(alert_tool["inputSchema"].is_object());
    
    // Verify input schema structure
    let input_schema = &alert_tool["inputSchema"];
    assert_eq!(input_schema["type"], "object");
    assert!(input_schema["properties"].is_object());
    assert!(input_schema["properties"]["limit"].is_object());

    Ok(())
}

#[tokio::test]
async fn test_get_wazuh_alert_summary_success() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::new();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Call the tool
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_wazuh_alert_summary",
            "arguments": {
                "limit": 2
            }
        }
    });

    let response = mcp_server.send_and_receive(&tool_call)?;

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["result"].is_object());
    
    let result = &response["result"];
    assert!(result["content"].is_array());
    assert_eq!(result["isError"], false);
    
    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty());
    
    // Verify content format
    for item in content {
        assert_eq!(item["type"], "text");
        assert!(item["text"].is_string());
        
        let text = item["text"].as_str().unwrap();
        assert!(text.contains("Alert ID:"));
        assert!(text.contains("Time:"));
        assert!(text.contains("Agent:"));
        assert!(text.contains("Level:"));
        assert!(text.contains("Description:"));
    }

    Ok(())
}

#[tokio::test]
async fn test_get_wazuh_alert_summary_empty_results() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::with_empty_alerts();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Call the tool
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_wazuh_alert_summary",
            "arguments": {}
        }
    });

    let response = mcp_server.send_and_receive(&tool_call)?;

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    
    let result = &response["result"];
    assert!(result["content"].is_array());
    assert_eq!(result["isError"], false);
    
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    assert_eq!(content[0]["text"], "No Wazuh alerts found.");

    Ok(())
}

#[tokio::test]
async fn test_get_wazuh_alert_summary_api_error() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::with_alerts_error();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Call the tool
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_wazuh_alert_summary",
            "arguments": {
                "limit": 5
            }
        }
    });

    let response = mcp_server.send_and_receive(&tool_call)?;

    // Verify error response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    
    let result = &response["result"];
    assert!(result["content"].is_array());
    assert_eq!(result["isError"], true);
    
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"], "text");
    
    let error_text = content[0]["text"].as_str().unwrap();
    assert!(error_text.contains("Error retrieving alerts from Wazuh"));

    Ok(())
}

#[tokio::test]
async fn test_invalid_tool_call() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::new();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Call non-existent tool
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "non_existent_tool",
            "arguments": {}
        }
    });

    let response = mcp_server.send_and_receive(&tool_call)?;

    // Should get an error response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["error"].is_object());

    Ok(())
}

#[tokio::test]
async fn test_parameter_validation() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::new();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Test with invalid parameter type (string instead of number)
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_wazuh_alert_summary",
            "arguments": {
                "limit": "invalid"
            }
        }
    });

    let response = mcp_server.send_and_receive(&tool_call)?;

    // Should get an error response for invalid parameters
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    // The response might be an error or a successful response with error content
    // depending on how rmcp handles parameter validation
    assert!(response["error"].is_object() || 
            (response["result"]["isError"] == true));

    Ok(())
}

#[tokio::test]
async fn test_malformed_alert_data_handling() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TEST_MUTEX.lock().unwrap();
    
    let mock_server = MockWazuhServer::with_malformed_alerts();
    let mut mcp_server = McpServerProcess::start_with_mock_wazuh(&mock_server)?;
    
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
    mcp_server.send_and_receive(&init_request)?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    mcp_server.send_message(&initialized)?;

    // Call the tool
    let tool_call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "get_wazuh_alert_summary",
            "arguments": {
                "limit": 5
            }
        }
    });

    let response = mcp_server.send_and_receive(&tool_call)?;

    // Should handle malformed data gracefully
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    
    let result = &response["result"];
    assert!(result["content"].is_array());
    // Should not error out, but handle missing fields gracefully
    assert_eq!(result["isError"], false);
    
    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty());
    
    // Verify that missing fields are handled with defaults
    for item in content {
        assert_eq!(item["type"], "text");
        let text = item["text"].as_str().unwrap();
        // Should contain default values for missing fields
        assert!(text.contains("Alert ID:"));
        assert!(text.contains("Unknown") || text.contains("missing_fields") || text.contains("partial_data"));
    }

    Ok(())
}
