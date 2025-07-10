//! Wazuh Manager rule tools
//! 
//! This module contains tools for retrieving and analyzing Wazuh security rules
//! from the Wazuh Manager.

use rmcp::{
    Error as McpError,
    model::{CallToolResult, Content},
    tool,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use wazuh_client::RulesClient;
use super::ToolModule;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetRulesSummaryParams {
    #[schemars(description = "Maximum number of rules to retrieve (default: 300)")]
    pub limit: Option<u32>,
    #[schemars(description = "Rule level to filter by (optional)")]
    pub level: Option<u32>,
    #[schemars(description = "Rule group to filter by (optional)")]
    pub group: Option<String>,
    #[schemars(description = "Filename to filter by (optional)")]
    pub filename: Option<String>,
}

#[derive(Clone)]
pub struct RuleTools {
    rules_client: Arc<Mutex<RulesClient>>,
}

impl RuleTools {
    pub fn new(rules_client: Arc<Mutex<RulesClient>>) -> Self {
        Self { rules_client }
    }

    #[tool(
        name = "get_wazuh_rules_summary",
        description = "Retrieves a summary of Wazuh security rules. Returns formatted rule information including ID, level, description, and groups. Supports filtering by level, group, and filename."
    )]
    pub async fn get_wazuh_rules_summary(
        &self,
        #[tool(aggr)] params: GetRulesSummaryParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);
        
        tracing::info!(
            limit = %limit, 
            level = ?params.level, 
            group = ?params.group, 
            filename = ?params.filename, 
            "Retrieving Wazuh rules summary"
        );

        let mut rules_client = self.rules_client.lock().await;

        match rules_client.get_rules(
            Some(limit),
            None, // offset
            params.level,
            params.group.as_deref(),
            params.filename.as_deref(),
        ).await {
            Ok(rules) => {
                if rules.is_empty() {
                    tracing::info!("No Wazuh rules found matching criteria. Returning standard message.");
                    return Self::not_found_result("Wazuh rules matching the specified criteria");
                }

                let num_rules = rules.len();
                let mcp_content_items: Vec<Content> = rules
                    .into_iter()
                    .map(|rule| {
                        let groups_str = rule.groups.join(", ");
                        
                        let compliance_info = {
                            let mut compliance = Vec::new();
                            if let Some(gdpr) = &rule.gdpr {
                                if !gdpr.is_empty() {
                                    compliance.push(format!("GDPR: {}", gdpr.join(", ")));
                                }
                            }
                            if let Some(hipaa) = &rule.hipaa {
                                if !hipaa.is_empty() {
                                    compliance.push(format!("HIPAA: {}", hipaa.join(", ")));
                                }
                            }
                            if let Some(pci) = &rule.pci_dss {
                                if !pci.is_empty() {
                                    compliance.push(format!("PCI DSS: {}", pci.join(", ")));
                                }
                            }
                            if let Some(nist) = &rule.nist_800_53 {
                                if !nist.is_empty() {
                                    compliance.push(format!("NIST 800-53: {}", nist.join(", ")));
                                }
                            }
                            if compliance.is_empty() {
                                String::new()
                            } else {
                                format!("\nCompliance: {}", compliance.join(" | "))
                            }
                        };

                        let severity = match rule.level {
                            0..=3 => "Low",
                            4..=7 => "Medium", 
                            8..=12 => "High",
                            13..=15 => "Critical",
                            _ => "Unknown",
                        };

                        let formatted_text = format!(
                            "Rule ID: {}\nLevel: {} ({})\nDescription: {}\nGroups: {}\nFile: {}\nStatus: {}{}",
                            rule.id, 
                            rule.level, 
                            severity,
                            rule.description, 
                            groups_str,
                            rule.filename,
                            rule.status,
                            compliance_info
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!("Successfully processed {} rules into {} MCP content items", num_rules, mcp_content_items.len());
                Self::success_result(mcp_content_items)
            }
            Err(e) => {
                let err_msg = Self::format_error("Manager", "retrieving rules", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }
}

impl ToolModule for RuleTools {}

