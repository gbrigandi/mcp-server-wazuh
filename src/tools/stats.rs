//! Stats tools for Wazuh MCP Server
//! 
//! This module contains tools for retrieving various statistics from Wazuh components,
//! including manager logs, remoted daemon stats, log collector stats, and weekly statistics.

use super::{ToolModule, ToolUtils};
use reqwest::StatusCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::Error as McpError;
use std::sync::Arc;
use tokio::sync::Mutex;
use wazuh_client::{ClusterClient, LogsClient};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchManagerLogsParams {
    #[schemars(description = "Maximum number of log entries to retrieve (default: 300)")]
    pub limit: Option<u32>,
    #[schemars(description = "Number of log entries to skip (default: 0)")]
    pub offset: Option<u32>,
    #[schemars(description = "Log level to filter by (e.g., \"error\", \"warning\", \"info\")")]
    pub level: String,
    #[schemars(description = "Log tag to filter by (e.g., \"wazuh-modulesd\") (optional)")]
    pub tag: Option<String>,
    #[schemars(description = "Search term to filter log descriptions (optional)")]
    pub search_term: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetManagerErrorLogsParams {
    #[schemars(description = "Maximum number of error log entries to retrieve (default: 300)")]
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetLogCollectorStatsParams {
    #[schemars(
        description = "Agent ID to get log collector stats for (required, e.g., \"0\", \"1\", \"001\")"
    )]
    pub agent_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetRemotedStatsParams {
    // No parameters needed for remoted stats
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetWeeklyStatsParams {
    // No parameters needed for weekly stats
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetClusterHealthParams {
    // No parameters needed for cluster health
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetClusterNodesParams {
    #[schemars(
        description = "Maximum number of nodes to retrieve (optional, Wazuh API default is 500)"
    )]
    pub limit: Option<u32>,
    #[schemars(description = "Number of nodes to skip (offset) (optional, default: 0)")]
    pub offset: Option<u32>,
    #[schemars(description = "Filter by node type (e.g., 'master', 'worker') (optional)")]
    pub node_type: Option<String>,
}

#[derive(Clone)]
pub struct StatsTools {
    logs_client: Arc<Mutex<LogsClient>>,
    cluster_client: Arc<Mutex<ClusterClient>>,
}

impl StatsTools {
    pub fn new(
        logs_client: Arc<Mutex<LogsClient>>,
        cluster_client: Arc<Mutex<ClusterClient>>,
    ) -> Self {
        Self {
            logs_client,
            cluster_client,
        }
    }

    pub async fn search_wazuh_manager_logs(
        &self,
        params: SearchManagerLogsParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);
        let offset = params.offset.unwrap_or(0);

        tracing::info!(
            limit = %limit,
            offset = %offset,
            level = ?params.level,
            tag = ?params.tag,
            search_term = ?params.search_term,
            "Searching Wazuh manager logs"
        );

        let mut logs_client = self.logs_client.lock().await;

        match logs_client
            .get_manager_logs(
                Some(limit),
                Some(offset),
                Some(&params.level),
                params.tag.as_deref(),
                params.search_term.as_deref(),
            )
            .await
        {
            Ok(log_entries) => {
                if log_entries.is_empty() {
                    tracing::info!("No Wazuh manager logs found matching criteria. Returning standard message.");
                    return Self::not_found_result("Wazuh manager logs");
                }

                let num_logs = log_entries.len();
                let mcp_content_items: Vec<Content> = log_entries
                    .into_iter()
                    .map(|log_entry| {
                        let formatted_text = format!(
                            "Timestamp: {}\nTag: {}\nLevel: {}\nDescription: {}",
                            log_entry.timestamp,
                            log_entry.tag,
                            log_entry.level,
                            log_entry.description
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} manager log entries into {} MCP content items",
                    num_logs,
                    mcp_content_items.len()
                );
                Self::success_result(mcp_content_items)
            }
            Err(e) => {
                let err_msg = Self::format_error("Manager", "searching logs", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_manager_error_logs(
        &self,
        params: GetManagerErrorLogsParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);

        tracing::info!(limit = %limit, "Retrieving Wazuh manager error logs");

        let mut logs_client = self.logs_client.lock().await;

        match logs_client.get_error_logs(Some(limit)).await {
            Ok(log_entries) => {
                if log_entries.is_empty() {
                    tracing::info!(
                        "No Wazuh manager error logs found. Returning standard message."
                    );
                    return Self::not_found_result("Wazuh manager error logs");
                }

                let num_logs = log_entries.len();
                let mcp_content_items: Vec<Content> = log_entries
                    .into_iter()
                    .map(|log_entry| {
                        let formatted_text = format!(
                            "Timestamp: {}\nTag: {}\nLevel: {}\nDescription: {}",
                            log_entry.timestamp,
                            log_entry.tag,
                            log_entry.level,
                            log_entry.description
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} manager error log entries into {} MCP content items",
                    num_logs,
                    mcp_content_items.len()
                );
                Self::success_result(mcp_content_items)
            }
            Err(e) => {
                let err_msg = Self::format_error("Manager", "retrieving error logs", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_log_collector_stats(
        &self,
        params: GetLogCollectorStatsParams,
    ) -> Result<CallToolResult, McpError> {
        let agent_id = match ToolUtils::format_agent_id(&params.agent_id) {
            Ok(formatted_id) => formatted_id,
            Err(err_msg) => {
                tracing::error!(
                    "Error formatting agent_id for log collector stats: {}",
                    err_msg
                );
                return Self::error_result(err_msg);
            }
        };

        tracing::info!(agent_id = %agent_id, "Retrieving Wazuh log collector stats");

        let mut logs_client = self.logs_client.lock().await;

        match logs_client.get_logcollector_stats(&agent_id).await {
            Ok(stats) => {
                // Helper closure to format a LogCollectorPeriod
                let format_period = |period_name: &str,
                                     period_data: &wazuh_client::logs::LogCollectorPeriod|
                 -> String {
                    let files_info: String = period_data
                        .files
                        .iter()
                        .map(|file: &wazuh_client::logs::LogFile| {
                            let targets_str: String = file
                                .targets
                                .iter()
                                .map(|target: &wazuh_client::logs::LogTarget| {
                                    format!(
                                        "        - Name: {}, Drops: {}",
                                        target.name, target.drops
                                    )
                                })
                                .collect::<Vec<String>>()
                                .join("\n");
                            let targets_display = if targets_str.is_empty() {
                                "        (No specific targets with drops for this file)".to_string()
                            } else {
                                format!("      Targets:\n{}", targets_str)
                            };
                            format!(
                                "    - Location: {}\n      Events: {}\n      Bytes: {}\n{}",
                                file.location, file.events, file.bytes, targets_display
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("\n\n");

                    let files_display = if files_info.is_empty() {
                        "    (No files processed in this period)".to_string()
                    } else {
                        files_info
                    };

                    format!(
                        "{}:\n  Start: {}\n  End: {}\n  Files:\n{}",
                        period_name, period_data.start, period_data.end, files_display
                    )
                };

                let global_period_info = format_period("Global Period", &stats.global);
                let interval_period_info = format_period("Interval Period", &stats.interval);

                let formatted_text = format!(
                    "Log Collector Stats for Agent: {}\n\n{}\n\n{}",
                    agent_id,
                    global_period_info,
                    interval_period_info
                );

                tracing::info!(
                    "Successfully retrieved and formatted log collector stats for agent {}",
                    agent_id
                );
                Self::success_result(vec![Content::text(formatted_text)])
            }
            Err(e) => {
                // Check if the error is due to agent not found or stats not available
                if let wazuh_client::WazuhApiError::ApiError(msg) = &e {
                    if msg.contains(&format!(
                        "Log collector stats for agent {} not found",
                        agent_id
                    )) {
                        tracing::info!("No log collector stats found for agent {} (API error). Returning standard message.", agent_id);
                        return Self::success_result(vec![Content::text(
                            format!("No log collector stats found for agent {}. The agent might not exist, stats are unavailable, or the agent is not active.", agent_id),
                        )]);
                    }
                    if msg.contains("Agent Not Found") {
                        tracing::info!(
                            "Agent {} not found (API error). Returning standard message.",
                            agent_id
                        );
                        return Self::success_result(vec![Content::text(format!(
                            "Agent {} not found. Cannot retrieve log collector stats.",
                            agent_id
                        ))]);
                    }
                }
                if let wazuh_client::WazuhApiError::HttpError { status, .. } = &e {
                    if *status == StatusCode::NOT_FOUND {
                        tracing::info!("No log collector stats found for agent {} (HTTP 404). Agent might not exist or endpoint unavailable.", agent_id);
                        return Self::success_result(vec![Content::text(
                           format!("No log collector stats found for agent {}. The agent might not exist, stats are unavailable, or the agent is not active.", agent_id),
                       )]);
                    }
                }
                let err_msg = format!(
                    "Error retrieving log collector stats for agent {} from Wazuh: {}",
                    agent_id, e
                );
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_remoted_stats(
        &self,
        _params: GetRemotedStatsParams,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Retrieving Wazuh remoted stats");

        let mut logs_client = self.logs_client.lock().await;

        match logs_client.get_remoted_stats().await {
            Ok(stats) => {
                let formatted_text = format!(
                    "Wazuh Remoted Statistics:\nQueue Size: {}\nTotal Queue Size: {}\nTCP Sessions: {}\nControl Message Count: {}\nDiscarded Message Count: {}\nMessages Sent (Bytes): {}\nBytes Received: {}\nDequeued After Close: {}",
                    stats.queue_size,
                    stats.total_queue_size,
                    stats.tcp_sessions,
                    stats.ctrl_msg_count,
                    stats.discarded_count,
                    stats.sent_bytes,
                    stats.recv_bytes,
                    stats.dequeued_after_close
                );

                tracing::info!("Successfully retrieved remoted stats");
                Self::success_result(vec![Content::text(formatted_text)])
            }
            Err(e) => {
                let err_msg = Self::format_error("Manager", "retrieving remoted stats", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_weekly_stats(
        &self,
        _params: GetWeeklyStatsParams,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Retrieving Wazuh weekly stats");

        let mut logs_client = self.logs_client.lock().await;

        match logs_client.get_weekly_stats().await {
            Ok(stats_value) => match serde_json::to_string_pretty(&stats_value) {
                Ok(formatted_json) => {
                    tracing::info!("Successfully retrieved and formatted weekly stats.");
                    Self::success_result(vec![Content::text(formatted_json)])
                }
                Err(e) => {
                    let err_msg = format!("Error formatting weekly stats JSON: {}", e);
                    tracing::error!("{}", err_msg);
                    Self::error_result(err_msg)
                }
            },
            Err(e) => {
                let err_msg = Self::format_error("Manager", "retrieving weekly stats", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_cluster_health(
        &self,
        _params: GetClusterHealthParams,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Retrieving Wazuh cluster health");

        let mut cluster_client = self.cluster_client.lock().await;

        match cluster_client.is_cluster_healthy().await {
            Ok(is_healthy) => {
                let health_status_text = if is_healthy {
                    "Cluster is healthy: Yes".to_string()
                } else {
                    // To provide more context, we can fetch the basic status
                    match cluster_client.get_cluster_status().await {
                        Ok(status) => {
                            let mut reasons = Vec::new();
                            if !status.enabled.eq_ignore_ascii_case("yes") {
                                reasons.push("cluster is not enabled");
                            }
                            if !status.running.eq_ignore_ascii_case("yes") {
                                reasons.push("cluster is not running");
                            }
                            if status.enabled.eq_ignore_ascii_case("yes") && status.running.eq_ignore_ascii_case("yes") {
                                 match cluster_client.get_cluster_healthcheck().await {
                                    Ok(hc) if hc.n_connected_nodes == 0 => reasons.push("no nodes are connected"),
                                    Err(_) => reasons.push("healthcheck endpoint failed or reported issues"),
                                    _ => {} // Healthy implies connected nodes
                                 }
                            }
                            if reasons.is_empty() && !is_healthy {
                                reasons.push("unknown reason, check detailed logs or healthcheck endpoint");
                            }
                            format!("Cluster is healthy: No. Reasons: {}", reasons.join("; "))
                        }
                        Err(_) => {
                            "Cluster is healthy: No. Additionally, failed to retrieve basic cluster status for more details.".to_string()
                        }
                    }
                };
                tracing::info!(
                    "Successfully retrieved cluster health: {}",
                    health_status_text
                );
                Self::success_result(vec![Content::text(health_status_text)])
            }
            Err(e) => {
                let err_msg = Self::format_error("Cluster", "retrieving health", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_cluster_nodes(
        &self,
        params: GetClusterNodesParams,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            limit = ?params.limit,
            offset = ?params.offset,
            node_type = ?params.node_type,
            "Retrieving Wazuh cluster nodes"
        );

        let mut cluster_client = self.cluster_client.lock().await;

        match cluster_client
            .get_cluster_nodes(params.limit, params.offset, params.node_type.as_deref())
            .await
        {
            Ok(nodes) => {
                if nodes.is_empty() {
                    tracing::info!("No Wazuh cluster nodes found matching criteria. Returning standard message.");
                    return Self::not_found_result("Wazuh cluster nodes");
                }

                let num_nodes = nodes.len();
                let mcp_content_items: Vec<Content> = nodes
                    .into_iter()
                    .map(|node| {
                        let status_indicator = match node.status.to_lowercase().as_str() {
                            "connected" | "active" => "🟢 CONNECTED",
                            "disconnected" => "🔴 DISCONNECTED",
                            _ => &node.status,
                        };
                        let formatted_text = format!(
                            "Node Name: {}\nType: {}\nVersion: {}\nIP: {}\nStatus: {}",
                            node.name, node.node_type, node.version, node.ip, status_indicator
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} cluster nodes into {} MCP content items",
                    num_nodes,
                    mcp_content_items.len()
                );
                Self::success_result(mcp_content_items)
            }
            Err(e) => {
                let err_msg = Self::format_error("Cluster", "retrieving nodes", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }
}

impl ToolModule for StatsTools {}

