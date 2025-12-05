//! Mock Wazuh API server for testing
//! 
//! This module provides a configurable mock server that simulates the Wazuh Indexer API
//! for testing purposes. It supports various response scenarios including success,
//! empty results, and error conditions.

use httpmock::prelude::*;
use serde_json::json;

pub struct MockWazuhServer {
    server: MockServer,
}

impl MockWazuhServer {
    pub fn new() -> Self {
        let server = MockServer::start();
        
        // Setup default authentication endpoint
        server.mock(|when, then| {
            when.method(POST).path("/security/user/authenticate");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jwt": "mock.jwt.token.eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
                }));
        });

        server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/wazuh-alerts.*/_search").unwrap());
            then.status(200)
                .header("content-type", "application/json")
                .json_body(Self::sample_alerts_response());
        });

        Self { server }
    }

    pub fn with_empty_alerts() -> Self {
        let server = MockServer::start();
        
        server.mock(|when, then| {
            when.method(POST).path("/security/user/authenticate");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jwt": "mock.jwt.token"
                }));
        });

        server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/wazuh-alerts.*/_search").unwrap());
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "hits": {
                        "hits": []
                    }
                }));
        });

        Self { server }
    }

    pub fn with_auth_error() -> Self {
        let server = MockServer::start();
        
        server.mock(|when, then| {
            when.method(POST).path("/security/user/authenticate");
            then.status(401)
                .header("content-type", "application/json")
                .json_body(json!({
                    "error": "Invalid credentials"
                }));
        });

        Self { server }
    }

    pub fn with_alerts_error() -> Self {
        let server = MockServer::start();
        
        server.mock(|when, then| {
            when.method(POST).path("/security/user/authenticate");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jwt": "mock.jwt.token"
                }));
        });

        server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/wazuh-alerts.*/_search").unwrap());
            then.status(500)
                .header("content-type", "application/json")
                .json_body(json!({
                    "error": "Internal server error"
                }));
        });

        Self { server }
    }

    pub fn with_malformed_alerts() -> Self {
        let server = MockServer::start();
        
        server.mock(|when, then| {
            when.method(POST).path("/security/user/authenticate");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "jwt": "mock.jwt.token"
                }));
        });

        server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/wazuh-alerts.*/_search").unwrap());
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "hits": {
                        "hits": [
                            {
                                "_source": {
                                    "id": "missing_fields",
                                    "timestamp": "invalid-date-format"
                                    // Missing rule, agent, etc.
                                }
                            },
                            {
                                "_source": {
                                    "id": "partial_data",
                                    "timestamp": "2024-01-15T10:30:45.123Z",
                                    "rule": {
                                        "level": 5
                                        // Missing description
                                    },
                                    "agent": {
                                        "id": "001"
                                        // Missing name
                                    }
                                }
                            }
                        ]
                    }
                }));
        });

        Self { server }
    }

    pub fn url(&self) -> String {
        self.server.url("")
    }

    pub fn host(&self) -> String {
        let url = self.url();
        let parts: Vec<&str> = url.trim_start_matches("http://").split(':').collect();
        parts[0].to_string()
    }

    pub fn port(&self) -> u16 {
        let url = self.url();
        let parts: Vec<&str> = url.trim_start_matches("http://").split(':').collect();
        parts[1].parse().unwrap()
    }

    fn sample_alerts_response() -> serde_json::Value {
        json!({
            "hits": {
                "hits": [
                    {
                        "_source": {
                            "id": "1747091815.1212763",
                            "timestamp": "2024-01-15T10:30:45.123Z",
                            "rule": {
                                "level": 7,
                                "description": "Attached USB Storage",
                                "groups": ["usb", "pci_dss"]
                            },
                            "agent": {
                                "id": "001",
                                "name": "web-server-01"
                            },
                            "data": {
                                "device": "/dev/sdb1",
                                "mount_point": "/media/usb"
                            }
                        }
                    },
                    {
                        "_source": {
                            "id": "1747066333.1207112",
                            "timestamp": "2024-01-15T10:25:12.456Z",
                            "rule": {
                                "level": 5,
                                "description": "New dpkg (Debian Package) installed.",
                                "groups": ["package_management", "debian"]
                            },
                            "agent": {
                                "id": "002",
                                "name": "database-server"
                            },
                            "data": {
                                "package": "nginx",
                                "version": "1.18.0-6ubuntu14.4"
                            }
                        }
                    },
                    {
                        "_source": {
                            "id": "1747055444.1205998",
                            "timestamp": "2024-01-15T10:20:33.789Z",
                            "rule": {
                                "level": 12,
                                "description": "Multiple authentication failures",
                                "groups": ["authentication_failed", "pci_dss"]
                            },
                            "agent": {
                                "id": "003",
                                "name": "ssh-gateway"
                            },
                            "data": {
                                "source_ip": "192.168.1.100",
                                "user": "admin",
                                "attempts": 5
                            }
                        }
                    }
                ]
            }
        })
    }
}

impl Default for MockWazuhServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_creation() {
        let mock_server = MockWazuhServer::new();
        assert!(!mock_server.url().is_empty());
        assert!(!mock_server.host().is_empty());
        assert!(mock_server.port() > 0);
    }

    #[tokio::test]
    async fn test_mock_server_auth_endpoint() {
        let mock_server = MockWazuhServer::new();
        let client = reqwest::Client::new();
        
        let response = client
            .post(format!("{}/security/user/authenticate", mock_server.url()))
            .json(&json!({"username": "admin", "password": "admin"}))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.get("jwt").is_some());
    }

    #[tokio::test]
    async fn test_mock_server_alerts_endpoint() {
        let mock_server = MockWazuhServer::new();
        let client = reqwest::Client::new();
        
        let response = client
            .post(format!("{}/wazuh-alerts*/_search", mock_server.url()))
            .json(&json!({"query": {"match_all": {}}}))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.get("hits").is_some());
        let hits = body["hits"]["hits"].as_array().unwrap();
        assert!(!hits.is_empty());
    }

    #[tokio::test]
    async fn test_empty_alerts_server() {
        let mock_server = MockWazuhServer::with_empty_alerts();
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/wazuh-alerts*/_search", mock_server.url()))
            .json(&json!({"query": {"match_all": {}}}))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        let hits = body["hits"]["hits"].as_array().unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn test_auth_error_server() {
        let mock_server = MockWazuhServer::with_auth_error();
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/security/user/authenticate", mock_server.url()))
            .json(&json!({"username": "admin", "password": "wrong"}))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn test_alerts_error_server() {
        let mock_server = MockWazuhServer::with_alerts_error();
        let client = reqwest::Client::new();

        let response = client
            .post(format!("{}/wazuh-alerts*/_search", mock_server.url()))
            .json(&json!({"query": {"match_all": {}}}))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 500);
    }
}
