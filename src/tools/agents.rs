//! Agent tools module for Wazuh MCP Server
//! 
//! This module contains all agent-related tool implementations including:
//! - Agent listing and filtering
//! - Agent process monitoring
//! - Agent network port monitoring

use super::{ToolModule, ToolUtils};
use reqwest::StatusCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::Error as McpError;
use std::sync::Arc;
use tokio::sync::Mutex;
use wazuh_client::{AgentsClient, Port as WazuhPort, VulnerabilityClient};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAgentsParams {
    #[schemars(description = "Maximum number of agents to retrieve (default: 300)")]
    pub limit: Option<u32>,
    #[schemars(
        description = "Agent status filter (active, disconnected, pending, never_connected)"
    )]
    pub status: String,
    #[schemars(description = "Agent name to search for (optional)")]
    pub name: Option<String>,
    #[schemars(description = "Agent IP address to filter by (optional)")]
    pub ip: Option<String>,
    #[schemars(description = "Agent group to filter by (optional)")]
    pub group: Option<String>,
    #[schemars(description = "Operating system platform to filter by (optional)")]
    pub os_platform: Option<String>,
    #[schemars(description = "Agent version to filter by (optional)")]
    pub version: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAgentProcessesParams {
    #[schemars(
        description = "Agent ID to get processes for (required, e.g., \"0\", \"1\", \"001\")"
    )]
    pub agent_id: String,
    #[schemars(description = "Maximum number of processes to retrieve (default: 300)")]
    pub limit: Option<u32>,
    #[schemars(description = "Search string to filter processes by name or command (optional)")]
    pub search: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAgentPortsParams {
    #[schemars(
        description = "Agent ID to get network ports for (required, e.g., \"001\", \"002\", \"003\")"
    )]
    pub agent_id: String,
    #[schemars(description = "Maximum number of ports to retrieve (default: 300)")]
    pub limit: Option<u32>,
    #[schemars(description = "Protocol to filter by (e.g., \"tcp\", \"udp\")")]
    pub protocol: String,
    #[schemars(description = "State to filter by (e.g., \"LISTENING\", \"ESTABLISHED\")")]
    pub state: String,
}

#[derive(Clone)]
pub struct AgentTools {
    agents_client: Arc<Mutex<AgentsClient>>,
    vulnerability_client: Arc<Mutex<VulnerabilityClient>>,
}

impl AgentTools {
    pub fn new(
        agents_client: Arc<Mutex<AgentsClient>>,
        vulnerability_client: Arc<Mutex<VulnerabilityClient>>,
    ) -> Self {
        Self {
            agents_client,
            vulnerability_client,
        }
    }

    pub async fn get_wazuh_agents(
        &self,
        params: GetAgentsParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);

        tracing::info!(
            limit = %limit,
            status = ?params.status,
            name = ?params.name,
            ip = ?params.ip,
            group = ?params.group,
            os_platform = ?params.os_platform,
            version = ?params.version,
            "Retrieving Wazuh agents"
        );

        let mut agents_client = self.agents_client.lock().await;

        match agents_client
            .get_agents(
                Some(limit),
                None, // offset
                None, // select
                None, // sort
                None, // search
                Some(&params.status),
                None, // query
                None, // older_than
                params.os_platform.as_deref(),
                None, // os_version
                None, // os_name
                None, // manager_host
                params.version.as_deref(),
                params.group.as_deref(),
                None, // node_name
                params.name.as_deref(),
                params.ip.as_deref(),
                None, // register_ip
                None, // group_config_status
                None, // distinct
            )
            .await
        {
            Ok(agents) => {
                if agents.is_empty() {
                    tracing::info!(
                        "No Wazuh agents found matching criteria. Returning standard message."
                    );
                    return Self::not_found_result(&format!(
                        "Wazuh agents matching the specified criteria (status: {})",
                        &params.status
                    ));
                }

                let num_agents = agents.len();
                let mcp_content_items: Vec<Content> = agents
                    .into_iter()
                    .map(|agent| {
                        let status_indicator = match agent.status.to_lowercase().as_str() {
                            "active" => "🟢 ACTIVE",
                            "disconnected" => "🔴 DISCONNECTED",
                            "pending" => "🟡 PENDING",
                            "never_connected" => "⚪ NEVER CONNECTED",
                            _ => &agent.status,
                        };

                        let ip_info = if let Some(ip) = &agent.ip {
                            format!("\nIP: {}", ip)
                        } else {
                            String::new()
                        };

                        let register_ip_info = if let Some(register_ip) = &agent.register_ip {
                            if agent.ip.as_ref() != Some(register_ip) {
                                format!("\nRegistered IP: {}", register_ip)
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        let os_info = if let Some(os) = &agent.os {
                            let mut os_parts = Vec::new();
                            if let Some(name) = &os.name {
                                os_parts.push(name.clone());
                            }
                            if let Some(version) = &os.version {
                                os_parts.push(version.clone());
                            }
                            if let Some(arch) = &os.arch {
                                os_parts.push(format!("({})", arch));
                            }
                            if !os_parts.is_empty() {
                                format!("\nOS: {}", os_parts.join(" "))
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        let version_info = if let Some(version) = &agent.version {
                            format!("\nAgent Version: {}", version)
                        } else {
                            String::new()
                        };

                        let group_info = if let Some(groups) = &agent.group {
                            if !groups.is_empty() {
                                format!("\nGroups: {}", groups.join(", "))
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

                        let last_keep_alive_info =
                            if let Some(last_keep_alive) = &agent.last_keep_alive {
                                format!("\nLast Keep Alive: {}", last_keep_alive)
                            } else {
                                String::new()
                            };

                        let date_add_info = if let Some(date_add) = &agent.date_add {
                            format!("\nRegistered: {}", date_add)
                        } else {
                            String::new()
                        };

                        let node_info = if let Some(node_name) = &agent.node_name {
                            format!("\nNode: {}", node_name)
                        } else {
                            String::new()
                        };

                        let config_status_info =
                            if let Some(config_status) = &agent.group_config_status {
                                let config_indicator = match config_status.to_lowercase().as_str() {
                                    "synced" => "✅ SYNCED",
                                    "not synced" => "❌ NOT SYNCED",
                                    _ => config_status,
                                };
                                format!("\nConfig Status: {}", config_indicator)
                            } else {
                                String::new()
                            };

                        let agent_id_display = if agent.id == "000" {
                            format!("{} (Wazuh Manager)", agent.id)
                        } else {
                            agent.id.clone()
                        };

                        let formatted_text = format!(
                            "Agent ID: {}\nName: {}\nStatus: {}{}{}{}{}{}{}{}{}{}",
                            agent_id_display,
                            agent.name,
                            status_indicator,
                            ip_info,
                            register_ip_info,
                            os_info,
                            version_info,
                            group_info,
                            last_keep_alive_info,
                            date_add_info,
                            node_info,
                            config_status_info
                        );
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} agents into {} MCP content items",
                    num_agents,
                    mcp_content_items.len()
                );
                Self::success_result(mcp_content_items)
            }
            Err(e) => {
                let err_msg = Self::format_error("agents", "retrieving agents", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    pub async fn get_wazuh_agent_processes(
        &self,
        params: GetAgentProcessesParams,
    ) -> Result<CallToolResult, McpError> {
        let agent_id = match ToolUtils::format_agent_id(&params.agent_id) {
            Ok(formatted_id) => formatted_id,
            Err(err_msg) => {
                tracing::error!("Error formatting agent_id for agent processes: {}", err_msg);
                return Self::error_result(err_msg);
            }
        };
        let limit = params.limit.unwrap_or(300);
        let offset = 0;

        tracing::info!(
            agent_id = %agent_id,
            limit = %limit,
            search = ?params.search,
            "Retrieving Wazuh agent processes"
        );

        let mut vulnerability_client = self.vulnerability_client.lock().await;

        match vulnerability_client
            .get_agent_processes(
                &agent_id,
                Some(limit),
                Some(offset),
                params.search.as_deref(),
            )
            .await
        {
            Ok(processes) => {
                if processes.is_empty() {
                    tracing::info!("No processes found for agent {} with current filters. Returning standard message.", agent_id);
                    return Self::not_found_result(&format!(
                        "processes for agent {} matching the specified criteria",
                        agent_id
                    ));
                }

                let num_processes = processes.len();
                let mcp_content_items: Vec<Content> = processes
                    .into_iter()
                    .map(|process| {
                        let mut details = vec![
                            format!("PID: {}", process.pid),
                            format!("Name: {}", process.name),
                        ];

                        if let Some(state) = &process.state {
                            details.push(format!("State: {}", state));
                        }
                        if let Some(ppid) = &process.ppid {
                            details.push(format!("PPID: {}", ppid));
                        }
                        if let Some(euser) = &process.euser {
                            details.push(format!("User: {}", euser));
                        }
                        if let Some(cmd) = &process.cmd {
                            details.push(format!("Command: {}", cmd));
                        }
                        if let Some(start_time_str) = &process.start_time {
                            if let Ok(start_time_unix) = start_time_str.parse::<i64>() {
                                // Assuming start_time is a Unix timestamp in seconds
                                use chrono::DateTime;
                                if let Some(dt) = DateTime::from_timestamp(start_time_unix, 0) {
                                    details.push(format!(
                                        "Start Time: {}",
                                        dt.format("%Y-%m-%d %H:%M:%S UTC")
                                    ));
                                } else {
                                    details.push(format!("Start Time: {} (raw)", start_time_str));
                                }
                            } else {
                                // If it's not a simple number, print as is
                                details.push(format!("Start Time: {}", start_time_str));
                            }
                        }
                        if let Some(resident_mem) = process.resident {
                            details.push(format!("Memory (Resident): {} KB", resident_mem / 1024));
                            // Assuming resident is in bytes
                        }
                        if let Some(vm_size) = process.vm_size {
                            details.push(format!("Memory (VM Size): {} KB", vm_size / 1024));
                            // Assuming vm_size is in bytes
                        }

                        Content::text(details.join("\n"))
                    })
                    .collect();

                tracing::info!(
                    "Successfully processed {} processes for agent {} into {} MCP content items",
                    num_processes,
                    agent_id,
                    mcp_content_items.len()
                );
                Self::success_result(mcp_content_items)
            }
            Err(e) => match e {
                wazuh_client::WazuhApiError::HttpError {
                    status,
                    message: _,
                    url: _,
                } if status == StatusCode::NOT_FOUND => {
                    tracing::info!("No process data found for agent {}. Syscollector might not have run or data is unavailable.", agent_id);
                    Self::success_result(vec![Content::text(
                            format!("No process data found for agent {}. The agent might not exist, syscollector data might be unavailable, or the agent is not active.", agent_id),
                        )])
                }
                _ => {
                    let err_msg = Self::format_error(
                        "agent processes",
                        &format!("retrieving processes for agent {}", agent_id),
                        &e,
                    );
                    tracing::error!("{}", err_msg);
                    Self::error_result(err_msg)
                }
            },
        }
    }

    pub async fn get_wazuh_agent_ports(
        &self,
        params: GetAgentPortsParams,
    ) -> Result<CallToolResult, McpError> {
        let agent_id = match ToolUtils::format_agent_id(&params.agent_id) {
            Ok(formatted_id) => formatted_id,
            Err(err_msg) => {
                tracing::error!("Error formatting agent_id for agent ports: {}", err_msg);
                return Self::error_result(err_msg);
            }
        };
        let limit = params.limit.unwrap_or(300);
        let offset = 0; // Default offset

        tracing::info!(
            agent_id = %agent_id,
            limit = %limit,
            protocol = ?params.protocol,
            state = ?params.state,
            "Retrieving Wazuh agent network ports"
        );

        let mut vulnerability_client = self.vulnerability_client.lock().await;

        // Note: The wazuh_client::VulnerabilityClient::get_agent_ports provided in the prompt
        // only supports filtering by protocol. If state filtering is needed, the client would need an update.
        // For now, we pass params.protocol and ignore params.state for the API call,
        // but we can filter by state client-side if necessary, or acknowledge this limitation.
        // The current wazuh-client `get_agent_ports` does not support state filtering directly in its parameters.
        // We will filter client-side for now if `params.state` is provided.
        match vulnerability_client
            .get_agent_ports(
                &agent_id,
                Some(limit * 2), // Fetch more to allow for client-side state filtering
                Some(offset),
                Some(&params.protocol),
            )
            .await
        {
            Ok(mut ports) => {
                let requested_state_is_listening =
                    params.state.trim().eq_ignore_ascii_case("listening");

                ports.retain(|port| {
                    tracing::debug!(
                        "Pre-filter port: {:?} (State: {:?}), requested_state_is_listening: {}",
                        port.inode, // Using inode for a concise port identifier in log
                        port.state,
                        requested_state_is_listening
                    );
                    let result = match port.state.as_ref().map(|s| s.trim()) {
                        Some(actual_port_state_str) => {
                            // Port has a state string
                            if actual_port_state_str.is_empty() {
                                // Filter out ports where state is present but an empty string
                                false
                            } else if requested_state_is_listening {
                                // User requested "listening": keep only if actual state is "listening"
                                actual_port_state_str.eq_ignore_ascii_case("listening")
                            } else {
                                // User requested non-"listening": keep if actual state is not "listening"
                                !actual_port_state_str.eq_ignore_ascii_case("listening")
                            }
                        }
                        None => {
                            // Port has no state (port.state is None)
                            if requested_state_is_listening {
                                // If user wants "listening" ports, a port with no state is not a match.
                                false
                            } else {
                                // If user wants non-"listening" ports, a port with no state is a match.
                                true
                            }
                        }
                    };

                    tracing::debug!(
                        "Post-filter decision for port: {:?}, Keep: {}",
                        port.inode,
                        result
                    );
                    result
                });

                // Apply limit after client-side filtering
                ports.truncate(limit as usize);

                if ports.is_empty() {
                    tracing::info!("No network ports found for agent {} with current filters. Returning standard message.", agent_id);
                    return Self::not_found_result(&format!(
                        "network ports for agent {} matching the specified criteria",
                        agent_id
                    ));
                }

                let num_ports = ports.len();
                let mcp_content_items: Vec<Content> = ports
                    .into_iter()
                    .map(|port: WazuhPort| {
                        // Explicitly type port
                        let mut details = vec![
                            format!("Protocol: {}", port.protocol),
                            format!(
                                "Local: {}:{}",
                                port.local.ip.clone().unwrap_or("N/A".to_string()),
                                port.local.port
                            ),
                        ];

                        if let Some(remote) = &port.remote {
                            details.push(format!(
                                "Remote: {}:{}",
                                remote.ip.clone().unwrap_or("N/A".to_string()),
                                remote.port
                            ));
                        }
                        if let Some(state) = &port.state {
                            details.push(format!("State: {}", state));
                        }
                        if let Some(process_name) = &port.process {
                            // process field in WazuhPort is Option<String>
                            details.push(format!("Process Name: {}", process_name));
                        }
                        if let Some(pid) = port.pid {
                            // pid field in WazuhPort is Option<u32>
                            details.push(format!("PID: {}", pid));
                        }
                        if let Some(inode) = port.inode {
                            details.push(format!("Inode: {}", inode));
                        }
                        if let Some(tx_queue) = port.tx_queue {
                            details.push(format!("TX Queue: {}", tx_queue));
                        }
                        if let Some(rx_queue) = port.rx_queue {
                            details.push(format!("RX Queue: {}", rx_queue));
                        }

                        Content::text(details.join("\n"))
                    })
                    .collect();

                tracing::info!("Successfully processed {} network ports for agent {} into {} MCP content items", num_ports, agent_id, mcp_content_items.len());
                Self::success_result(mcp_content_items)
            }
            Err(e) => match e {
                wazuh_client::WazuhApiError::HttpError {
                    status,
                    message: _,
                    url: _,
                } if status == StatusCode::NOT_FOUND => {
                    tracing::info!("No network port data found for agent {}. Syscollector might not have run or data is unavailable.", agent_id);
                    Self::success_result(vec![Content::text(
                            format!("No network port data found for agent {}. The agent might not exist, syscollector data might be unavailable, or the agent is not active.", agent_id),
                        )])
                }
                _ => {
                    let err_msg = Self::format_error(
                        "agent ports",
                        &format!("retrieving network ports for agent {}", agent_id),
                        &e,
                    );
                    tracing::error!("{}", err_msg);
                    Self::error_result(err_msg)
                }
            },
        }
    }
}

impl ToolModule for AgentTools {}

