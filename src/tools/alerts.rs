//! Wazuh Indexer alert tools
//! 
//! This module contains tools for retrieving and analyzing Wazuh security alerts
//! from the Wazuh Indexer.

use rmcp::{
    Error as McpError,
    model::{CallToolResult, Content},
    tool,
};
use std::sync::Arc;
use wazuh_client::WazuhIndexerClient;
use super::ToolModule;

/// Parameters for getting alert summary
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAlertSummaryParams {
    #[schemars(description = "Maximum number of alerts to retrieve (default: 300)")]
    pub limit: Option<u32>,
}

/// Alert tools implementation
#[derive(Clone)]
pub struct AlertTools {
    indexer_client: Arc<WazuhIndexerClient>,
}

impl AlertTools {
    pub fn new(indexer_client: Arc<WazuhIndexerClient>) -> Self {
        Self { indexer_client }
    }

    #[tool(
        name = "get_wazuh_alert_summary",
        description = "Retrieves a summary of Wazuh security alerts. Returns formatted alert information including ID, timestamp, and description."
    )]
    pub async fn get_wazuh_alert_summary(
        &self,
        #[tool(aggr)] params: GetAlertSummaryParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);
        
        tracing::info!(limit = %limit, "Retrieving Wazuh alert summary");

        match self.indexer_client.get_alerts(Some(limit)).await {
            Ok(raw_alerts) => {
                if raw_alerts.is_empty() {
                    tracing::info!("No Wazuh alerts found to process. Returning standard message.");
                    return Self::not_found_result("Wazuh alerts");
                }

                let num_alerts_to_process = raw_alerts.len();
                let mcp_content_items: Vec<Content> = raw_alerts
                    .into_iter()
                    .map(|alert_value| {
                        let source = alert_value.get("_source").unwrap_or(&alert_value);

                        let id = source.get("id")
                            .and_then(|v| v.as_str())
                            .or_else(|| alert_value.get("_id").and_then(|v| v.as_str()))
                            .unwrap_or("Unknown ID");

                        let description = source.get("rule")
                            .and_then(|r| r.get("description"))
                            .and_then(|d| d.as_str())
                            .unwrap_or("No description available");

                        let timestamp = source.get("timestamp")
                            .and_then(|t| t.as_str())
                            .unwrap_or("Unknown time");

                        let agent_name = source.get("agent")
                            .and_then(|a| a.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown agent");

                        let rule_level = source.get("rule")
                            .and_then(|r| r.get("level"))
                            .and_then(|l| l.as_u64())
                            .unwrap_or(0);

                        let formatted_text = format!(
                            "Alert ID: {}\nTime: {}\nAgent: {}\nLevel: {}\nDescription: {}",
                            id, timestamp, agent_name, rule_level, description
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!("Successfully processed {} alerts into {} MCP content items", num_alerts_to_process, mcp_content_items.len());
                Self::success_result(mcp_content_items)
            }
            Err(e) => {
                let err_msg = Self::format_error("Indexer", "retrieving alerts", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }
}

impl ToolModule for AlertTools {}

