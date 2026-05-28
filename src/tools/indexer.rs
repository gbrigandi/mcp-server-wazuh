//! Wazuh Indexer generic query tools
//!
//! This module provides a flexible search tool that accepts raw Elasticsearch
//! query DSL, enabling custom alert queries, aggregations, and field selection
//! across any Wazuh index pattern.

use rmcp::{
    ErrorData as McpError,
    model::{CallToolResult, Content},
    tool,
};
use std::sync::Arc;
use wazuh_client::WazuhIndexerClient;

use super::ToolModule;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetIndexerQueryParams {
    #[schemars(description = "Index pattern to search (default: wazuh-alerts-*)")]
    pub index: Option<String>,

    #[schemars(description = "Elasticsearch query DSL as JSON (e.g. {\"match_all\":{}}, {\"bool\":{\"must\":[...]}})")]
    pub query: serde_json::Value,

    #[schemars(description = "Maximum number of hits to return (default: 100, max: 10000)")]
    pub size: Option<u32>,

    #[schemars(description = "Elasticsearch aggregations DSL as JSON (e.g. {\"by_level\":{\"terms\":{\"field\":\"rule.level\"}}})")]
    pub aggregations: Option<serde_json::Value>,

    #[schemars(description = "List of fields to include in _source (e.g. [\"timestamp\",\"agent.name\",\"rule.description\"])")]
    pub source_fields: Option<Vec<String>>,

    #[schemars(description = "Track total hit count accurately (default: true)")]
    pub track_total: Option<bool>,
}

/// Normalize a Value that might arrive as a JSON string into a proper object.
fn ensure_object(value: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::String(s) = &value {
        serde_json::from_str(s).unwrap_or(value)
    } else {
        value
    }
}

#[derive(Clone)]
pub struct IndexerTools {
    indexer_client: Arc<WazuhIndexerClient>,
}

impl IndexerTools {
    pub fn new(indexer_client: Arc<WazuhIndexerClient>) -> Self {
        Self { indexer_client }
    }

    #[tool(
        name = "get_wazuh_indexer_query",
        description = "Execute a raw Elasticsearch query against the Wazuh Indexer. Accepts ES query DSL JSON for flexible search across wazuh-alerts-* and other indices. Supports aggregations for reporting (e.g. group by agent, MITRE tactic, rule level). Returns hits, total count, and aggregation buckets as JSON."
    )]
    pub async fn get_wazuh_indexer_query(
        &self,
        params: GetIndexerQueryParams,
    ) -> Result<CallToolResult, McpError> {
        let index = params.index.unwrap_or_else(|| "wazuh-alerts-*".to_string());
        let size = params.size.map(|s| s.min(10000)).unwrap_or(100);
        let query = ensure_object(params.query);
        let aggregations = params.aggregations.map(ensure_object);

        tracing::info!(
            index = %index,
            size = %size,
            has_aggs = aggregations.is_some(),
            "Executing custom Wazuh Indexer query"
        );

        match self
            .indexer_client
            .search(
                &index,
                query,
                Some(size),
                aggregations,
                params.source_fields,
                params.track_total,
            )
            .await
        {
            Ok(response) => {
                let formatted =
                    serde_json::to_string_pretty(&response).unwrap_or_default();
                tracing::info!("Successfully executed Indexer query, returning results");
                Self::success_result(vec![Content::text(formatted)])
            }
            Err(e) => {
                let err_msg = Self::format_error("Indexer", "executing query", &e);
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }
}

impl ToolModule for IndexerTools {}
