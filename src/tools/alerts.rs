//! Wazuh Indexer alert tools
//!
//! This module contains tools for retrieving and analyzing Wazuh security alerts
//! from the Wazuh Indexer.

use reqwest::Method;
use rmcp::{
    ErrorData as McpError,
    model::{CallToolResult, Content},
    tool,
};
use serde_json::{json, Value};
use std::sync::Arc;
use wazuh_client::WazuhIndexerClient;
use super::ToolModule;

/// Parameters for getting alert summary.
///
/// All filter fields are optional and additive: when none of them are
/// provided the tool falls back to the existing `get_alerts(limit)` path,
/// preserving the v0.2.5 behaviour. When any filter is set the underlying
/// query becomes a `bool` query against `wazuh-alerts*/_search` with the
/// matching clauses populated.
///
/// See https://github.com/gbrigandi/mcp-server-wazuh/issues/22.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAlertSummaryParams {
    #[schemars(description = "Maximum number of alerts to retrieve (default: 300)")]
    pub limit: Option<u32>,
    #[schemars(description = "Lower bound on alert timestamp, ISO 8601 (e.g. \"2026-02-10T00:00:00Z\"). Inclusive.")]
    pub date_from: Option<String>,
    #[schemars(description = "Upper bound on alert timestamp, ISO 8601 (e.g. \"2026-02-17T23:59:59Z\"). Inclusive.")]
    pub date_to: Option<String>,
    #[schemars(description = "Filter to alerts from this Wazuh agent ID (e.g. \"001\").")]
    pub agent_id: Option<String>,
    #[schemars(description = "Filter to alerts from this Wazuh agent name (e.g. \"web-prod-01\"). Matched on `agent.name`.")]
    pub agent_name: Option<String>,
    #[schemars(description = "Filter to alerts from a specific Wazuh rule ID (e.g. \"5710\").")]
    pub rule_id: Option<String>,
    #[schemars(description = "Filter to alerts whose rule level is greater-or-equal to this value (1-15).")]
    pub rule_level_min: Option<u8>,
}

impl GetAlertSummaryParams {
    /// Returns true if any filter beyond `limit` was supplied. Used to
    /// decide between the legacy `get_alerts(limit)` fast path and the
    /// custom OpenSearch query path.
    fn has_filters(&self) -> bool {
        self.date_from.is_some()
            || self.date_to.is_some()
            || self.agent_id.is_some()
            || self.agent_name.is_some()
            || self.rule_id.is_some()
            || self.rule_level_min.is_some()
    }

    /// Builds the OpenSearch query body for `wazuh-alerts*/_search` based
    /// on which filters are populated. Each filter is only added as a
    /// clause when its corresponding parameter is `Some`, matching the
    /// shape suggested in #22.
    ///
    /// Time bounds end up in a `range` clause on `timestamp`. Agent /
    /// rule fields go in `term` (or `match` for `agent.name`, which is
    /// often a text field) clauses. The minimum-rule-level filter is a
    /// half-open range, so it stays compatible with future
    /// `rule_level_max` if anyone wants to add a band filter later.
    pub(crate) fn to_opensearch_query(&self, limit: u32) -> Value {
        let mut must: Vec<Value> = Vec::new();

        if self.date_from.is_some() || self.date_to.is_some() {
            let mut range = serde_json::Map::new();
            if let Some(from) = self.date_from.as_ref() {
                range.insert("gte".to_string(), Value::String(from.clone()));
            }
            if let Some(to) = self.date_to.as_ref() {
                range.insert("lte".to_string(), Value::String(to.clone()));
            }
            must.push(json!({ "range": { "timestamp": Value::Object(range) } }));
        }

        if let Some(id) = self.agent_id.as_ref() {
            must.push(json!({ "term": { "agent.id": id } }));
        }
        if let Some(name) = self.agent_name.as_ref() {
            // `agent.name` is typically a text field in Wazuh's index
            // mapping; `match` works whether it's text or keyword while
            // `term` would only hit on a `.keyword` subfield.
            must.push(json!({ "match": { "agent.name": name } }));
        }
        if let Some(rule_id) = self.rule_id.as_ref() {
            must.push(json!({ "term": { "rule.id": rule_id } }));
        }
        if let Some(min) = self.rule_level_min {
            must.push(json!({ "range": { "rule.level": { "gte": min } } }));
        }

        let query: Value = if must.is_empty() {
            json!({ "match_all": {} })
        } else {
            json!({ "bool": { "must": must } })
        };

        json!({
            "size": limit,
            "sort": [{ "timestamp": { "order": "desc" } }],
            "query": query,
        })
    }
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
        description = "Retrieves a summary of Wazuh security alerts. Supports optional time-range, agent, and rule filters; when none are provided returns the most recent N alerts."
    )]
    pub async fn get_wazuh_alert_summary(
        &self,
        params: GetAlertSummaryParams,
    ) -> Result<CallToolResult, McpError> {
        let limit = params.limit.unwrap_or(300);

        tracing::info!(
            limit = %limit,
            has_filters = params.has_filters(),
            date_from = ?params.date_from,
            date_to = ?params.date_to,
            agent_id = ?params.agent_id,
            agent_name = ?params.agent_name,
            rule_id = ?params.rule_id,
            rule_level_min = ?params.rule_level_min,
            "Retrieving Wazuh alert summary"
        );

        // Result of the alert lookup: either the existing fast path
        // (no filters → call wazuh-client's stock get_alerts) or a
        // custom OpenSearch query when any filter is set. Both paths
        // return the same `Vec<Value>` shape so the formatting code
        // below stays unchanged.
        let raw_alerts: Result<Vec<Value>, String> = if params.has_filters() {
            self.fetch_alerts_with_filters(&params, limit).await
        } else {
            self.indexer_client
                .get_alerts(Some(limit))
                .await
                .map_err(|e| Self::format_error("Indexer", "retrieving alerts", &e))
        };

        match raw_alerts {
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

                        // Extract source IP from data.srcip (common for SSH, network alerts)
                        let src_ip = source.get("data")
                            .and_then(|d| d.get("srcip"))
                            .and_then(|ip| ip.as_str())
                            .or_else(|| source.get("data")
                                .and_then(|d| d.get("src_ip"))
                                .and_then(|ip| ip.as_str()))
                            .unwrap_or("");

                        // Extract destination IP if available
                        let dst_ip = source.get("data")
                            .and_then(|d| d.get("dstip"))
                            .and_then(|ip| ip.as_str())
                            .or_else(|| source.get("data")
                                .and_then(|d| d.get("dst_ip"))
                                .and_then(|ip| ip.as_str()))
                            .unwrap_or("");

                        // Extract source user if available
                        let src_user = source.get("data")
                            .and_then(|d| d.get("srcuser"))
                            .and_then(|u| u.as_str())
                            .or_else(|| source.get("data")
                                .and_then(|d| d.get("dstuser"))
                                .and_then(|u| u.as_str()))
                            .unwrap_or("");

                        // Build formatted text with optional fields
                        let mut formatted_text = format!(
                            "Alert ID: {}\nTime: {}\nAgent: {}\nLevel: {}\nDescription: {}",
                            id, timestamp, agent_name, rule_level, description
                        );

                        if !src_ip.is_empty() {
                            formatted_text.push_str(&format!("\nSource IP: {}", src_ip));
                        }
                        if !dst_ip.is_empty() {
                            formatted_text.push_str(&format!("\nDestination IP: {}", dst_ip));
                        }
                        if !src_user.is_empty() {
                            formatted_text.push_str(&format!("\nUser: {}", src_user));
                        }
                        Content::text(formatted_text)
                    })
                    .collect();

                tracing::info!("Successfully processed {} alerts into {} MCP content items", num_alerts_to_process, mcp_content_items.len());
                Self::success_result(mcp_content_items)
            }
            Err(err_msg) => {
                tracing::error!("{}", err_msg);
                Self::error_result(err_msg)
            }
        }
    }

    /// Filtered-fetch path: builds the OpenSearch query from
    /// [`GetAlertSummaryParams`] and POSTs it directly via the indexer
    /// client's `make_indexer_request`, then unwraps `hits.hits` the same
    /// way the upstream `get_alerts` helper does.
    ///
    /// Kept private to this module because it's an implementation detail
    /// of the filtered tool path. If a future tool grows similar needs we
    /// can pull this up to a shared helper.
    async fn fetch_alerts_with_filters(
        &self,
        params: &GetAlertSummaryParams,
        limit: u32,
    ) -> Result<Vec<Value>, String> {
        let endpoint = "/wazuh-alerts*/_search";
        let body = params.to_opensearch_query(limit);

        tracing::debug!(?body, "Sending filtered alert query to Wazuh Indexer");

        let response = self
            .indexer_client
            .make_indexer_request(Method::POST, endpoint, Some(body))
            .await
            .map_err(|e| Self::format_error("Indexer", "retrieving filtered alerts", &e))?;

        let hits = response
            .get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h_array| h_array.as_array())
            .ok_or_else(|| {
                "Indexer response missing 'hits.hits' array (filtered query)".to_string()
            })?;

        Ok(hits.iter().cloned().collect())
    }
}

impl ToolModule for AlertTools {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Helper to build a minimally-populated params struct quickly in tests.
    fn p() -> GetAlertSummaryParams {
        GetAlertSummaryParams {
            limit: None,
            date_from: None,
            date_to: None,
            agent_id: None,
            agent_name: None,
            rule_id: None,
            rule_level_min: None,
        }
    }

    #[test]
    fn has_filters_false_when_only_limit_set() {
        let mut params = p();
        params.limit = Some(50);
        assert!(!params.has_filters(), "limit alone is not a filter");
    }

    #[test]
    fn has_filters_true_for_each_filter_field() {
        let cases: Vec<Box<dyn Fn(&mut GetAlertSummaryParams)>> = vec![
            Box::new(|p| p.date_from = Some("2026-02-10T00:00:00Z".into())),
            Box::new(|p| p.date_to = Some("2026-02-17T23:59:59Z".into())),
            Box::new(|p| p.agent_id = Some("001".into())),
            Box::new(|p| p.agent_name = Some("web-prod-01".into())),
            Box::new(|p| p.rule_id = Some("5710".into())),
            Box::new(|p| p.rule_level_min = Some(10)),
        ];

        for setter in cases {
            let mut params = p();
            setter(&mut params);
            assert!(
                params.has_filters(),
                "expected has_filters=true after setting filter, got false: {:?}",
                params
            );
        }
    }

    #[test]
    fn query_with_no_filters_emits_match_all() {
        let body = p().to_opensearch_query(100);
        assert_eq!(body["query"], json!({ "match_all": {} }));
        assert_eq!(body["size"], 100);
        assert_eq!(body["sort"][0]["timestamp"]["order"], "desc");
    }

    #[test]
    fn query_with_time_range_emits_range_clause() {
        let mut params = p();
        params.date_from = Some("2026-02-10T00:00:00Z".into());
        params.date_to = Some("2026-02-17T23:59:59Z".into());

        let body = params.to_opensearch_query(50);
        let must = &body["query"]["bool"]["must"];
        assert_eq!(must.as_array().unwrap().len(), 1);
        assert_eq!(
            must[0],
            json!({
                "range": {
                    "timestamp": {
                        "gte": "2026-02-10T00:00:00Z",
                        "lte": "2026-02-17T23:59:59Z"
                    }
                }
            })
        );
    }

    #[test]
    fn query_with_only_date_from_omits_lte() {
        let mut params = p();
        params.date_from = Some("2026-02-10T00:00:00Z".into());
        let body = params.to_opensearch_query(50);
        let must = &body["query"]["bool"]["must"];
        let range = &must[0]["range"]["timestamp"];
        assert_eq!(range["gte"], "2026-02-10T00:00:00Z");
        assert!(range.get("lte").is_none(), "lte must be omitted when date_to is None");
    }

    #[test]
    fn query_with_agent_id_uses_term() {
        let mut params = p();
        params.agent_id = Some("001".into());
        let body = params.to_opensearch_query(50);
        assert_eq!(
            body["query"]["bool"]["must"][0],
            json!({ "term": { "agent.id": "001" } })
        );
    }

    #[test]
    fn query_with_agent_name_uses_match_not_term() {
        // `agent.name` is typically a text field; `match` works whether
        // the field is mapped as text or keyword. `term` would only hit
        // on a `.keyword` subfield, which isn't guaranteed to exist.
        let mut params = p();
        params.agent_name = Some("web-prod-01".into());
        let body = params.to_opensearch_query(50);
        let clause = &body["query"]["bool"]["must"][0];
        assert!(clause.get("match").is_some(), "expected match clause for agent.name");
        assert!(clause.get("term").is_none(), "term clause would miss text-mapped agent.name");
    }

    #[test]
    fn query_with_rule_level_min_uses_gte_range() {
        let mut params = p();
        params.rule_level_min = Some(10);
        let body = params.to_opensearch_query(50);
        assert_eq!(
            body["query"]["bool"]["must"][0],
            json!({ "range": { "rule.level": { "gte": 10 } } })
        );
    }

    #[test]
    fn query_combines_all_filters_into_must_array() {
        let params = GetAlertSummaryParams {
            limit: Some(150),
            date_from: Some("2026-02-10T00:00:00Z".into()),
            date_to: Some("2026-02-17T23:59:59Z".into()),
            agent_id: Some("001".into()),
            agent_name: Some("web-prod-01".into()),
            rule_id: Some("5710".into()),
            rule_level_min: Some(10),
        };
        let body = params.to_opensearch_query(150);
        let must = body["query"]["bool"]["must"]
            .as_array()
            .expect("must clause should be an array");
        // 1 range (timestamp) + agent.id + agent.name + rule.id + 1 range (rule.level) = 5
        assert_eq!(must.len(), 5, "expected one clause per active filter");
        assert_eq!(body["size"], 150);
        assert_eq!(body["sort"][0]["timestamp"]["order"], "desc");
    }
}

