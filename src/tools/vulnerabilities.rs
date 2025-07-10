//! Wazuh Manager vulnerability tools
//! 
//! This module contains tools for retrieving and analyzing vulnerability information
//! from the Wazuh Manager.

use rmcp::{
    Error as McpError,
    model::{CallToolResult, Content},
    schemars,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use wazuh_client::{VulnerabilityClient, VulnerabilitySeverity};
use super::ToolModule;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetVulnerabilitiesSummaryParams {
    #[schemars(description = "Maximum number of vulnerabilities to retrieve (default: 10000)")]
    pub limit: Option<u32>,
    #[schemars(description = "Agent ID to filter vulnerabilities by (required, e.g., \"0\", \"1\", \"001\")")]
    pub agent_id: String,
    #[schemars(description = "Severity level to filter by (Low, Medium, High, Critical) (optional)")]
    pub severity: Option<String>,
    #[schemars(description = "CVE ID to search for (optional)")]
    pub cve: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetCriticalVulnerabilitiesParams {
    #[schemars(description = "Agent ID to get critical vulnerabilities for (required, e.g., \"0\", \"1\", \"001\")")]
    pub agent_id: String,
    #[schemars(description = "Maximum number of vulnerabilities to retrieve (default: 300)")]
    pub limit: Option<u32>,
}

#[derive(Clone)]
pub struct VulnerabilityTools {
    vulnerability_client: Arc<Mutex<VulnerabilityClient>>,
}

impl VulnerabilityTools {
    pub fn new(vulnerability_client: Arc<Mutex<VulnerabilityClient>>) -> Self {
        Self { vulnerability_client }
    }

    fn format_agent_id(agent_id_str: &str) -> Result<String, String> {
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

    pub async fn get_wazuh_vulnerability_summary(
        &self,
        params: GetVulnerabilitiesSummaryParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(10000);
        let offset = 0; // Default offset, can be extended in future if needed

        let agent_id = match Self::format_agent_id(&params.agent_id) {
            Ok(formatted_id) => formatted_id,
            Err(err_msg) => {
                tracing::error!(
                    "Error formatting agent_id for vulnerability summary: {}",
                    err_msg
                );
                return Ok(CallToolResult::error(vec![Content::text(err_msg)]));
            }
        };

        tracing::info!(
            limit = %limit,
            agent_id = %agent_id,
            severity = ?params.severity,
            cve = ?params.cve,
            "Retrieving Wazuh vulnerability summary"
        );

        let mut vulnerability_client = self.vulnerability_client.lock().await;

        let vulnerabilities = vulnerability_client
            .get_agent_vulnerabilities(
                &agent_id,
                Some(limit),
                Some(offset),
                params
                    .severity
                    .as_deref()
                    .and_then(VulnerabilitySeverity::from_str),
            )
            .await;

        match vulnerabilities {
            Ok(vulnerabilities) => {
                if vulnerabilities.is_empty() {
                    tracing::info!("No Wazuh vulnerabilities found matching criteria. Returning standard message.");
                    return Ok(CallToolResult::success(vec![Content::text(
                        "No Wazuh vulnerabilities found matching the specified criteria.",
                    )]));
                }

                let num_vulnerabilities = vulnerabilities.len();
                let mcp_content_items: Vec<Content> = vulnerabilities
                    .into_iter()
                    .map(|vuln| {
                        let severity_indicator = match vuln.severity {
                            VulnerabilitySeverity::Critical => "🔴 CRITICAL",
                            VulnerabilitySeverity::High => "🟠 HIGH",
                            VulnerabilitySeverity::Medium => "🟡 MEDIUM",
                            VulnerabilitySeverity::Low => "🟢 LOW",
                        };

                        let published_info = if let Some(published) = &vuln.published {
                            format!("\nPublished: {}", published)
                        } else {
                            String::new()
                        };

                        let updated_info = if let Some(updated) = &vuln.updated {
                            format!("\nUpdated: {}", updated)
                        } else {
                            String::new()
                        };

                        let detection_time_info = if let Some(detection_time) = &vuln.detection_time
                        {
                            format!("\nDetection Time: {}", detection_time)
                        } else {
                            String::new()
                        };

                        let agent_info = {
                            let id_str = vuln.agent_id.as_deref();
                            let name_str = vuln.agent_name.as_deref();

                            match (id_str, name_str) {
                                (Some("000"), Some(name)) => {
                                    format!("\nAgent: {} (Wazuh Manager, ID: 000)", name)
                                }
                                (Some("000"), None) => {
                                    "\nAgent: Wazuh Manager (ID: 000)".to_string()
                                }
                                (Some(id), Some(name)) => format!("\nAgent: {} (ID: {})", name, id),
                                (Some(id), None) => format!("\nAgent ID: {}", id),
                                (None, Some(name)) => format!("\nAgent: {} (ID: Unknown)", name), // Should ideally not happen if ID is a primary key for agent context
                                (None, None) => String::new(), // No agent information available
                            }
                        };

                        let cvss_info = if let Some(cvss) = &vuln.cvss {
                            let mut cvss_parts = Vec::new();
                            if let Some(cvss2) = &cvss.cvss2 {
                                if let Some(score) = cvss2.base_score {
                                    cvss_parts.push(format!("CVSS2: {}", score));
                                }
                            }
                            if let Some(cvss3) = &cvss.cvss3 {
                                if let Some(score) = cvss3.base_score {
                                    cvss_parts.push(format!("CVSS3: {}", score));
                                }
                            }
                            if !cvss_parts.is_empty() {
                                format!("\nCVSS Scores: {}", cvss_parts.join(", "))
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        let reference_info = if let Some(reference) = &vuln.reference {
                            format!("\nReference: {}", reference)
                        } else {
                            String::new()
                        };

                        let description = vuln
                            .description
                            .as_deref()
                            .unwrap_or("No description available");

                        let formatted_text = format!(
                            "CVE: {}\nSeverity: {}\nTitle: {}\nDescription: {}{}{}{}{}{}{}",
                            vuln.cve,
                            severity_indicator,
                            vuln.title,
                            description,
                            published_info,
                            updated_info,
                            detection_time_info,
                            agent_info,
                            cvss_info,
                            reference_info
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} vulnerabilities into {} MCP content items",
                    num_vulnerabilities,
                    mcp_content_items.len()
                );
                Ok(CallToolResult::success(mcp_content_items))
            }
            Err(e) => {
                use reqwest::StatusCode;
                match e {
                    wazuh_client::WazuhApiError::HttpError {
                        status,
                        message: _,
                        url: _,
                    } if status == StatusCode::NOT_FOUND => {
                        tracing::info!("No vulnerability summary found for agent {}. Returning standard message.", agent_id);
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "No vulnerability summary found for agent {}.",
                            agent_id
                        ))]));
                    }
                    _ => {}
                }
                let err_msg = format!("Error retrieving vulnerabilities from Wazuh: {}", e);
                tracing::error!("{}", err_msg);
                Ok(CallToolResult::error(vec![Content::text(err_msg)]))
            }
        }
    }

    pub async fn get_wazuh_critical_vulnerabilities(
        &self,
        params: GetCriticalVulnerabilitiesParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);
        let agent_id = match Self::format_agent_id(&params.agent_id) {
            Ok(formatted_id) => formatted_id,
            Err(err_msg) => {
                tracing::error!(
                    "Error formatting agent_id for critical vulnerabilities: {}",
                    err_msg
                );
                return Ok(CallToolResult::error(vec![Content::text(err_msg)]));
            }
        };

        tracing::info!(
            agent_id = %agent_id,
            "Retrieving critical vulnerabilities for Wazuh agent"
        );

        let mut vulnerability_client = self.vulnerability_client.lock().await;

        match vulnerability_client
            .get_critical_vulnerabilities(&agent_id, Some(limit))
            .await
        {
            Ok(vulnerabilities) => {
                if vulnerabilities.is_empty() {
                    tracing::info!("No critical vulnerabilities found for agent {}. Returning standard message.", agent_id);
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "No critical vulnerabilities found for agent {}.",
                        agent_id
                    ))]));
                }

                let num_vulnerabilities = vulnerabilities.len();
                let mcp_content_items: Vec<Content> = vulnerabilities
                    .into_iter()
                    .map(|vuln| {
                        let published_info = if let Some(published) = &vuln.published {
                            format!("\nPublished: {}", published)
                        } else {
                            String::new()
                        };

                        let updated_info = if let Some(updated) = &vuln.updated {
                            format!("\nUpdated: {}", updated)
                        } else {
                            String::new()
                        };

                        let detection_time_info = if let Some(detection_time) = &vuln.detection_time {
                            format!("\nDetection Time: {}", detection_time)
                        } else {
                            String::new()
                        };

                        let agent_info = if let Some(agent_name) = &vuln.agent_name {
                            format!("\nAgent: {} (ID: {})", agent_name, vuln.agent_id.as_deref().unwrap_or("Unknown"))
                        } else if let Some(agent_id) = &vuln.agent_id {
                            format!("\nAgent ID: {}", agent_id)
                        } else {
                            String::new()
                        };

                        let cvss_info = if let Some(cvss) = &vuln.cvss {
                            let mut cvss_parts = Vec::new();
                            if let Some(cvss2) = &cvss.cvss2 {
                                if let Some(score) = cvss2.base_score {
                                    cvss_parts.push(format!("CVSS2: {}", score));
                                }
                            }
                            if let Some(cvss3) = &cvss.cvss3 {
                                if let Some(score) = cvss3.base_score {
                                    cvss_parts.push(format!("CVSS3: {}", score));
                                }
                            }
                            if !cvss_parts.is_empty() {
                                format!("\nCVSS Scores: {}", cvss_parts.join(", "))
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        let reference_info = if let Some(reference) = &vuln.reference {
                            format!("\nReference: {}", reference)
                        } else {
                            String::new()
                        };

                        let description = vuln.description.as_deref().unwrap_or("No description available");

                        let formatted_text = format!(
                            "🔴 CRITICAL VULNERABILITY\nCVE: {}\nTitle: {}\nDescription: {}{}{}{}{}{}{}",
                            vuln.cve,
                            vuln.title,
                            description,
                            published_info,
                            updated_info,
                            detection_time_info,
                            agent_info,
                            cvss_info,
                            reference_info
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} critical vulnerabilities into {} MCP content items",
                    num_vulnerabilities,
                    mcp_content_items.len()
                );
                Ok(CallToolResult::success(mcp_content_items))
            }
            Err(e) => {
                use reqwest::StatusCode;
                match e {
                    wazuh_client::WazuhApiError::HttpError {
                        status,
                        message: _,
                        url: _,
                    } if status == StatusCode::NOT_FOUND => {
                        tracing::info!("No critical vulnerabilities found for agent {}. Returning standard message.", agent_id);
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "No critical vulnerabilities found for agent {}.",
                            agent_id
                        ))]));
                    }
                    _ => {}
                }
                let err_msg = format!(
                    "Error retrieving critical vulnerabilities from Wazuh for agent {}: {}",
                    agent_id, e
                );
                tracing::error!("{}", err_msg);
                Ok(CallToolResult::error(vec![Content::text(err_msg)]))
            }
        }
    }
}

impl ToolModule for VulnerabilityTools {}