//
// Purpose:
//
// This Rust application implements an MCP (Model Context Protocol) server that acts as a
// bridge to a Wazuh instance. It exposes various Wazuh functionalities as tools that can
// be invoked by MCP clients (e.g., AI models, automation scripts).
//
// Architecture:
// The application follows a modular design where the main MCP server delegates to
// domain-specific tool modules, promoting separation of concerns and maintainability.
//
// Structure:
// - `main()`: Entry point of the application. Initializes logging (tracing),
//   sets up the `WazuhToolsServer`, and starts the MCP server using stdio or HTTP transport.
//
// - `WazuhToolsServer`: The core orchestrator struct that implements the `rmcp::ServerHandler` trait
//   and uses `#[tool_router]` and `#[tool_handler]` macros. It acts as a facade that delegates
//   tool calls to specialized domain modules:
//   - Holds instances of domain-specific tool modules (AgentTools, AlertTools, RuleTools, etc.)
//   - Its methods, decorated with `#[tool(...)]`, define the MCP tool interface and delegate
//     to the appropriate domain module for actual implementation
//   - Manages the lifecycle and configuration of Wazuh client connections
//
// - Domain-Specific Tool Modules (in `tools/` package):
//   - `AlertTools` (`tools/alerts.rs`): Handles alert-related operations via Wazuh Indexer
//   - `RuleTools` (`tools/rules.rs`): Manages security rule queries via Wazuh Manager API
//   - `VulnerabilityTools` (`tools/vulnerabilities.rs`): Processes vulnerability data via Wazuh Manager API
//   - `AgentTools` (`tools/agents.rs`): Handles agent management and system information queries
//   - `StatsTools` (`tools/stats.rs`): Provides logging, statistics, and cluster health monitoring
//
// Configuration:
// The server requires the following environment variables to connect to the Wazuh instance:
// - `WAZUH_API_HOST`: Hostname or IP address of the Wazuh API.
// - `WAZUH_API_PORT`: Port number for the Wazuh API (default: 55000).
// - `WAZUH_API_USERNAME`: Username for Wazuh API authentication.
// - `WAZUH_API_PASSWORD`: Password for Wazuh API authentication.
// - `WAZUH_INDEXER_HOST`: Hostname or IP address of the Wazuh Indexer.
// - `WAZUH_INDEXER_PORT`: Port number for the Wazuh Indexer API (default: 9200).
// - `WAZUH_INDEXER_USERNAME`: Username for Wazuh Indexer authentication.
// - `WAZUH_INDEXER_PASSWORD`: Password for Wazuh Indexer authentication.
// - `WAZUH_VERIFY_SSL`: Set to "true" to enable SSL certificate verification, "false" otherwise (default: false).
// - `WAZUH_TEST_PROTOCOL`: (Optional) Protocol to use for Wazuh API/Indexer connections, e.g., "http" or "https" (default: "https").
// Logging behavior is controlled by the `RUST_LOG` environment variable (e.g., `RUST_LOG=info,mcp_server_wazuh=debug`).

use clap::Parser;
use dotenv::dotenv;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
#[cfg(feature = "http")]
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use std::env;
use std::sync::Arc;

use wazuh_client::WazuhClientFactory;

mod tools;
use tools::agents::{AgentTools, GetAgentPortsParams, GetAgentProcessesParams, GetAgentsParams};
use tools::alerts::{AlertTools, GetAlertSummaryParams};
use tools::rules::{GetRulesSummaryParams, RuleTools};
use tools::stats::{
    GetClusterHealthParams, GetClusterNodesParams, GetLogCollectorStatsParams,
    GetManagerErrorLogsParams, GetRemotedStatsParams, GetWeeklyStatsParams,
    SearchManagerLogsParams, StatsTools,
};
use tools::vulnerabilities::{
    GetCriticalVulnerabilitiesParams, GetVulnerabilitiesSummaryParams, VulnerabilityTools,
};
use tools::indexer::{IndexerTools, GetIndexerQueryParams};

#[derive(Parser, Debug)]
#[command(name = "mcp-server-wazuh")]
#[command(about = "Wazuh SIEM MCP Server")]
struct Args {
    /// Transport mode: stdio or http
    #[arg(long, default_value = "stdio")]
    transport: String,

    /// HTTP server bind address (only for http transport)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// HTTP server port (only for http transport)
    #[arg(long, default_value = "8080")]
    port: u16,
}

#[derive(Clone)]
struct WazuhToolsServer {
    agent_tools: AgentTools,
    alert_tools: AlertTools,
    rule_tools: RuleTools,
    stats_tools: StatsTools,
    vulnerability_tools: VulnerabilityTools,
    indexer_tools: IndexerTools,
    tool_router: ToolRouter<Self>,
}

impl WazuhToolsServer {
    fn new() -> Result<Self, anyhow::Error> {
        dotenv().ok();

        let api_host = env::var("WAZUH_API_HOST").unwrap_or_else(|_| "localhost".to_string());
        let api_port: u16 = env::var("WAZUH_API_PORT")
            .unwrap_or_else(|_| "55000".to_string())
            .parse()
            .unwrap_or(55000);
        let api_username = env::var("WAZUH_API_USERNAME").unwrap_or_else(|_| "wazuh".to_string());
        let api_password = env::var("WAZUH_API_PASSWORD").unwrap_or_else(|_| "wazuh".to_string());

        let indexer_host =
            env::var("WAZUH_INDEXER_HOST").unwrap_or_else(|_| "localhost".to_string());
        let indexer_port: u16 = env::var("WAZUH_INDEXER_PORT")
            .unwrap_or_else(|_| "9200".to_string())
            .parse()
            .unwrap_or(9200);
        let indexer_username =
            env::var("WAZUH_INDEXER_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let indexer_password =
            env::var("WAZUH_INDEXER_PASSWORD").unwrap_or_else(|_| "admin".to_string());

        let verify_ssl = env::var("WAZUH_VERIFY_SSL")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let test_protocol = env::var("WAZUH_TEST_PROTOCOL")
            .ok()
            .or_else(|| Some("https".to_string()));

        let mut builder = WazuhClientFactory::builder()
            .api_host(api_host)
            .api_port(api_port)
            .api_credentials(&api_username, &api_password)
            .indexer_host(indexer_host)
            .indexer_port(indexer_port)
            .indexer_credentials(&indexer_username, &indexer_password)
            .verify_ssl(verify_ssl);

        if let Some(protocol) = test_protocol {
            builder = builder.protocol(protocol);
        }

        let wazuh_factory = builder.build();

        let wazuh_indexer_client = wazuh_factory.create_indexer_client();
        let wazuh_rules_client = wazuh_factory.create_rules_client();
        let wazuh_vulnerability_client = wazuh_factory.create_vulnerability_client();
        let wazuh_agents_client = wazuh_factory.create_agents_client();
        let wazuh_logs_client = wazuh_factory.create_logs_client();
        let wazuh_cluster_client = wazuh_factory.create_cluster_client();

        let indexer_client_arc = Arc::new(wazuh_indexer_client);
        let rules_client_arc = Arc::new(tokio::sync::Mutex::new(wazuh_rules_client));
        let vulnerability_client_arc =
            Arc::new(tokio::sync::Mutex::new(wazuh_vulnerability_client));
        let agents_client_arc = Arc::new(tokio::sync::Mutex::new(wazuh_agents_client));
        let logs_client_arc = Arc::new(tokio::sync::Mutex::new(wazuh_logs_client));
        let cluster_client_arc = Arc::new(tokio::sync::Mutex::new(wazuh_cluster_client));

        let agent_tools =
            AgentTools::new(agents_client_arc.clone(), vulnerability_client_arc.clone());
        let alert_tools = AlertTools::new(indexer_client_arc.clone());
        let rule_tools = RuleTools::new(rules_client_arc.clone());
        let stats_tools = StatsTools::new(logs_client_arc.clone(), cluster_client_arc.clone());
        let vulnerability_tools = VulnerabilityTools::new(vulnerability_client_arc.clone());
        let indexer_tools = IndexerTools::new(indexer_client_arc.clone());

        Ok(Self {
            agent_tools,
            alert_tools,
            rule_tools,
            stats_tools,
            vulnerability_tools,
            indexer_tools,
            tool_router: Self::tool_router(),
        })
    }
}

#[tool_router]
impl WazuhToolsServer {
    #[tool(
        name = "get_wazuh_alert_summary",
        description = "Retrieves a summary of Wazuh security alerts. Returns formatted alert information including ID, timestamp, and description."
    )]
    async fn get_wazuh_alert_summary(
        &self,
        Parameters(params): Parameters<GetAlertSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        self.alert_tools.get_wazuh_alert_summary(params).await
    }

    #[tool(
        name = "get_wazuh_rules_summary",
        description = "Retrieves a summary of Wazuh security rules. Returns formatted rule information including ID, level, description, and groups. Supports filtering by level, group, and filename."
    )]
    async fn get_wazuh_rules_summary(
        &self,
        Parameters(params): Parameters<GetRulesSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        self.rule_tools.get_wazuh_rules_summary(params).await
    }

    #[tool(
        name = "get_wazuh_vulnerability_summary",
        description = "Retrieves a summary of Wazuh vulnerability detections for a specific agent. Returns formatted vulnerability information including CVE ID, severity, detection time, and agent details. Supports filtering by severity level."
    )]
    async fn get_wazuh_vulnerability_summary(
        &self,
        Parameters(params): Parameters<GetVulnerabilitiesSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        self.vulnerability_tools
            .get_wazuh_vulnerability_summary(params)
            .await
    }

    #[tool(
        name = "get_wazuh_critical_vulnerabilities",
        description = "Retrieves critical vulnerabilities for a specific Wazuh agent. Returns formatted vulnerability information including CVE ID, title, description, CVSS scores, and detection details. Only shows vulnerabilities with 'Critical' severity level."
    )]
    async fn get_wazuh_critical_vulnerabilities(
        &self,
        Parameters(params): Parameters<GetCriticalVulnerabilitiesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.vulnerability_tools
            .get_wazuh_critical_vulnerabilities(params)
            .await
    }

    #[tool(
        name = "get_wazuh_agents",
        description = "Retrieves a list of Wazuh agents with their current status and details. Returns formatted agent information including ID, name, IP, status, OS details, and last activity. Supports filtering by status, name, IP, group, OS platform, and version."
    )]
    async fn get_wazuh_agents(
        &self,
        Parameters(params): Parameters<GetAgentsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.agent_tools.get_wazuh_agents(params).await
    }

    #[tool(
        name = "get_wazuh_agent_processes",
        description = "Retrieves a list of running processes for a specific Wazuh agent. Returns formatted process information including PID, name, state, user, and command. Supports filtering by process name/command."
    )]
    async fn get_wazuh_agent_processes(
        &self,
        Parameters(params): Parameters<GetAgentProcessesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.agent_tools.get_wazuh_agent_processes(params).await
    }

    #[tool(
        name = "get_wazuh_cluster_health",
        description = "Checks the health of the Wazuh cluster. Returns whether the cluster is enabled, running, and if nodes are connected."
    )]
    async fn get_wazuh_cluster_health(
        &self,
        Parameters(params): Parameters<GetClusterHealthParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.get_wazuh_cluster_health(params).await
    }

    #[tool(
        name = "get_wazuh_cluster_nodes",
        description = "Retrieves a list of nodes in the Wazuh cluster. Returns formatted node information including name, type, version, IP, and status. Supports filtering by limit, offset, and node type."
    )]
    async fn get_wazuh_cluster_nodes(
        &self,
        Parameters(params): Parameters<GetClusterNodesParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.get_wazuh_cluster_nodes(params).await
    }

    #[tool(
        name = "search_wazuh_manager_logs",
        description = "Searches Wazuh manager logs. Returns formatted log entries including timestamp, tag, level, and description. Supports filtering by limit, offset, level, tag, and a search term."
    )]
    async fn search_wazuh_manager_logs(
        &self,
        Parameters(params): Parameters<SearchManagerLogsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.search_wazuh_manager_logs(params).await
    }

    #[tool(
        name = "get_wazuh_manager_error_logs",
        description = "Retrieves Wazuh manager error logs. Returns formatted log entries including timestamp, tag, level (error), and description."
    )]
    async fn get_wazuh_manager_error_logs(
        &self,
        Parameters(params): Parameters<GetManagerErrorLogsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.get_wazuh_manager_error_logs(params).await
    }

    #[tool(
        name = "get_wazuh_log_collector_stats",
        description = "Retrieves log collector statistics for a specific Wazuh agent. Returns information about events processed, dropped, bytes, and target log files."
    )]
    async fn get_wazuh_log_collector_stats(
        &self,
        Parameters(params): Parameters<GetLogCollectorStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.get_wazuh_log_collector_stats(params).await
    }

    #[tool(
        name = "get_wazuh_remoted_stats",
        description = "Retrieves statistics from the Wazuh remoted daemon. Returns information about queue size, TCP sessions, event counts, and message traffic."
    )]
    async fn get_wazuh_remoted_stats(
        &self,
        Parameters(params): Parameters<GetRemotedStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.get_wazuh_remoted_stats(params).await
    }

    #[tool(
        name = "get_wazuh_agent_ports",
        description = "Retrieves a list of open network ports for a specific Wazuh agent. Returns formatted port information including local/remote IP and port, protocol, state, and associated process/PID. Supports filtering by protocol and state."
    )]
    async fn get_wazuh_agent_ports(
        &self,
        Parameters(params): Parameters<GetAgentPortsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.agent_tools.get_wazuh_agent_ports(params).await
    }

    #[tool(
        name = "get_wazuh_weekly_stats",
        description = "Retrieves weekly statistics from the Wazuh manager. Returns a JSON object detailing various metrics aggregated over the past week."
    )]
    async fn get_wazuh_weekly_stats(
        &self,
        Parameters(params): Parameters<GetWeeklyStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.stats_tools.get_wazuh_weekly_stats(params).await
    }

    #[tool(
        name = "get_wazuh_indexer_query",
        description = "Execute a raw Elasticsearch query against the Wazuh Indexer. Accepts ES query DSL JSON for flexible search across wazuh-alerts-* and other indices. Supports aggregations for reporting (e.g. group by agent, MITRE tactic, rule level). Returns hits, total count, and aggregation buckets as JSON."
    )]
    async fn get_wazuh_indexer_query(
        &self,
        Parameters(params): Parameters<GetIndexerQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        self.indexer_tools.get_wazuh_indexer_query(params).await
    }
}

#[tool_handler]
impl ServerHandler for WazuhToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            server_info: Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "This server provides tools to interact with a Wazuh SIEM instance for security monitoring and analysis."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Wazuh MCP Server...");

    match args.transport.as_str() {
        "stdio" => {
            tracing::info!("Using stdio transport");
            let server = WazuhToolsServer::new().expect("Error initializing Wazuh tools server");
            let service = server.serve(stdio()).await.inspect_err(|e| {
                tracing::error!("serving error: {:?}", e);
            })?;
            service.waiting().await?;
        }
        #[cfg(feature = "http")]
        "http" => {
            use axum::Router;

            tracing::info!("Starting HTTP server on {}:{}", args.host, args.port);
            let addr = format!("{}:{}", args.host, args.port);

            let service = StreamableHttpService::new(
                || Ok(WazuhToolsServer::new().expect("Error initializing Wazuh tools server")),
                Arc::new(LocalSessionManager::default()),
                StreamableHttpServerConfig::default(),
            );

            let router = Router::new().nest_service("/mcp", service);
            let tcp_listener = tokio::net::TcpListener::bind(&addr).await?;

            tracing::info!("Listening on http://{}/mcp", addr);

            axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async {
                    tokio::signal::ctrl_c().await.unwrap();
                    tracing::info!("Received Ctrl-C, shutting down...");
                })
                .await?;
        }
        #[cfg(not(feature = "http"))]
        "http" => {
            anyhow::bail!(
                "HTTP transport is not enabled. Rebuild with the 'http' feature: cargo build --features http"
            );
        }
        _ => {
            anyhow::bail!(
                "Unknown transport: '{}'. Use 'stdio' or 'http'",
                args.transport
            );
        }
    }

    Ok(())
}
