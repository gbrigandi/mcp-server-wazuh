//! Tools module for Wazuh MCP Server
//! 
//! This module contains all the tool implementations organized by Wazuh component domains.
//! Each submodule handles a specific area of Wazuh functionality.

pub mod agents;
pub mod alerts;
pub mod rules;
pub mod stats;
pub mod vulnerabilities;

use rmcp::model::{CallToolResult, Content};
use rmcp::Error as McpError;

pub trait ToolModule {
    fn format_error(component: &str, operation: &str, error: &dyn std::fmt::Display) -> String {
        format!("Error {} from Wazuh {}: {}", operation, component, error)
    }

    fn success_result(content: Vec<Content>) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(content))
    }

    fn error_result(message: String) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::error(vec![Content::text(message)]))
    }

    fn not_found_result(resource: &str) -> Result<CallToolResult, McpError> {
        let message = if resource == "Wazuh alerts" {
            "No Wazuh alerts found.".to_string()
        } else {
            format!("No {} found matching the specified criteria.", resource)
        };
        Ok(CallToolResult::success(vec![Content::text(message)]))
    }
}

pub struct ToolUtils;

impl ToolUtils {
    pub fn format_agent_id(agent_id_str: &str) -> Result<String, String> {
        // Attempt to parse as a number first
        if let Ok(num) = agent_id_str.parse::<u32>() {
            if num > 999 {
                Err(format!(
                    "Agent ID '{}' is too large. Must be a number between 0 and 999.",
                    agent_id_str
                ))
            } else {
                Ok(format!("{:03}", num))
            }
        } else if agent_id_str.len() == 3 && agent_id_str.chars().all(|c| c.is_ascii_digit()) {
            // Already correctly formatted (e.g., "001")
            Ok(agent_id_str.to_string())
        } else {
            Err(format!(
                "Invalid agent_id format: '{}'. Must be a number (e.g., 1, 12) or a 3-digit string (e.g., 001, 012).",
                agent_id_str
            ))
        }
    }
}

