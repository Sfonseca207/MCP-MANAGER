use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    ClaudeCode,
    Cursor,
    ClaudeDesktop,
    VsCode,
    Windsurf,
}

impl ClientType {
    pub fn all() -> &'static [ClientType] {
        &[
            ClientType::ClaudeCode,
            ClientType::Cursor,
            ClientType::ClaudeDesktop,
            ClientType::VsCode,
            ClientType::Windsurf,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ClientType::ClaudeCode => "claude_code",
            ClientType::Cursor => "cursor",
            ClientType::ClaudeDesktop => "claude_desktop",
            ClientType::VsCode => "vscode",
            ClientType::Windsurf => "windsurf",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ClientType::ClaudeCode => "Claude Code",
            ClientType::Cursor => "Cursor",
            ClientType::ClaudeDesktop => "Claude Desktop",
            ClientType::VsCode => "VS Code",
            ClientType::Windsurf => "Windsurf",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude_code" => Some(ClientType::ClaudeCode),
            "cursor" => Some(ClientType::Cursor),
            "claude_desktop" => Some(ClientType::ClaudeDesktop),
            "vscode" => Some(ClientType::VsCode),
            "windsurf" => Some(ClientType::Windsurf),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeType {
    Global,
    Project,
}

impl ScopeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScopeType::Global => "global",
            ScopeType::Project => "project",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "global" => Some(ScopeType::Global),
            "project" => Some(ScopeType::Project),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportType {
    Stdio,
    Http,
    Sse,
}

impl TransportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportType::Stdio => "stdio",
            TransportType::Http => "http",
            TransportType::Sse => "sse",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "stdio" => Some(TransportType::Stdio),
            "http" => Some(TransportType::Http),
            "sse" => Some(TransportType::Sse),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    AutoDetected,
    Manual,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::AutoDetected => "auto_detectado",
            SourceType::Manual => "manual",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "auto_detectado" => Some(SourceType::AutoDetected),
            "manual" => Some(SourceType::Manual),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpDefinition {
    pub name: String,
    pub transport_type: TransportType,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<BTreeMap<String, String>>,
}

impl McpDefinition {
    pub fn from_json_value(name: &str, value: &Value) -> crate::error::AppResult<Self> {
        let obj = value
            .as_object()
            .ok_or_else(|| format!("MCP '{name}' must be an object"))?;

        let explicit_type = obj.get("type").and_then(|v| v.as_str());
        let has_url = obj.contains_key("url") || obj.contains_key("serverUrl");

        let transport_type = match explicit_type {
            Some("http") => TransportType::Http,
            Some("sse") => TransportType::Sse,
            Some("stdio") => TransportType::Stdio,
            _ if has_url => {
                if obj.get("type").and_then(|v| v.as_str()) == Some("sse") {
                    TransportType::Sse
                } else {
                    TransportType::Http
                }
            }
            _ => TransportType::Stdio,
        };

        let command = obj.get("command").and_then(|v| v.as_str()).map(str::to_string);
        let args = obj.get("args").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
        });
        let env = parse_string_map(obj.get("env"));
        let url = obj
            .get("url")
            .or_else(|| obj.get("serverUrl"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let headers = parse_string_map(obj.get("headers"));

        Ok(Self {
            name: name.to_string(),
            transport_type,
            command,
            args,
            env,
            url,
            headers,
        })
    }

    pub fn to_json_value(&self) -> Value {
        let mut obj = serde_json::Map::new();

        match self.transport_type {
            TransportType::Stdio => {
                obj.insert("type".into(), Value::String("stdio".into()));
                if let Some(command) = &self.command {
                    obj.insert("command".into(), Value::String(command.clone()));
                }
                if let Some(args) = &self.args {
                    obj.insert(
                        "args".into(),
                        Value::Array(args.iter().map(|a| Value::String(a.clone())).collect()),
                    );
                }
                if let Some(env) = &self.env {
                    obj.insert("env".into(), string_map_to_value(env));
                }
            }
            TransportType::Http | TransportType::Sse => {
                obj.insert(
                    "type".into(),
                    Value::String(self.transport_type.as_str().into()),
                );
                if let Some(url) = &self.url {
                    obj.insert("url".into(), Value::String(url.clone()));
                }
                if let Some(headers) = &self.headers {
                    obj.insert("headers".into(), string_map_to_value(headers));
                }
            }
        }

        Value::Object(obj)
    }

    pub fn to_windsurf_json_value(&self) -> Value {
        let mut value = self.to_json_value();
        if let Some(obj) = value.as_object_mut() {
            if let Some(url) = obj.remove("url") {
                obj.insert("serverUrl".into(), url);
            }
        }
        value
    }

    pub fn estimated_tool_count(&self) -> u32 {
        match self.transport_type {
            TransportType::Stdio => {
                let args_len = self.args.as_ref().map(|a| a.len()).unwrap_or(0);
                (8 + args_len.min(5) as u32).min(20)
            }
            TransportType::Http | TransportType::Sse => 8,
        }
    }
}

fn parse_string_map(value: Option<&Value>) -> Option<BTreeMap<String, String>> {
    value.and_then(|v| v.as_object()).map(|obj| {
        obj.iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect()
    })
}

fn string_map_to_value(map: &BTreeMap<String, String>) -> Value {
    Value::Object(
        map.iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect(),
    )
}

pub fn parse_mcp_servers_map(value: &Value) -> crate::error::AppResult<BTreeMap<String, McpDefinition>> {
    let mut servers = BTreeMap::new();
    if let Some(obj) = value.as_object() {
        for (name, def) in obj {
            servers.insert(name.clone(), McpDefinition::from_json_value(name, def)?);
        }
    }
    Ok(servers)
}
