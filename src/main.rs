use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

/// Stateless MCP 2026-07-28 server for Bluesky + AT Protocol + music.vaked.dev.
/// Each request is self-contained. No sessions. Pure JSON-RPC over stdio.

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct ToolResult {
    content: Vec<ToolContent>,
}

#[derive(Debug, Serialize)]
struct ToolContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

fn list_tools() -> serde_json::Value {
    let mut listing = serde_json::json!({
       "tools": [
           {"name": "bluesky_post", "description": "Post to Bluesky"},
           {"name": "bluesky_timeline", "description": "Get home timeline"},
           {"name": "bluesky_profile", "description": "Get a profile"},
           {"name": "bluesky_search", "description": "Search posts"},
           {"name": "couchsky_profile", "description": "Public: get any profile"},
           {"name": "couchsky_feed", "description": "Public: get author feed"},
           {"name": "couchsky_search", "description": "Public: search posts"},
           {"name": "music_now_playing", "description": "Get now playing from music.vaked.dev"},
           {"name": "music_choreographies", "description": "List all choreographies"},
           {"name": "music_advance", "description": "Advance to next track in choreography"},
       ]
    });
    for tool in listing["tools"].as_array_mut().unwrap() {
        let name = tool["name"].as_str().unwrap();
        let description = if name == "music_now_playing" || name == "music_choreographies" {
            "Static sample data only; no live music service integration".to_string()
        } else {
            format!("Not implemented: {}", tool["description"].as_str().unwrap())
        };
        tool["description"] = serde_json::json!(description);
    }
    listing
}

fn handle_tool_call(name: &str, _args: &serde_json::Value) -> serde_json::Value {
    match name {
        "music_now_playing" => {
            let result = "Static sample (not live playback):\n8BIT-WRAITH ♫ The Wave and the Memory — MEM|8 · 1/1\n[The Architect] ♫ The Unforgiven II — Metallica · edesapa · 1/7";
            serde_json::json!({"content": [{"type": "text", "text": result}]})
        }
        "music_choreographies" => {
            let result = "Static sample (not a live service response):\nchoreographies:\n  edesapa (7 tracks, Metallica)\n  vivaldi-spring (3 tracks)\n  mem8 (1 track, 8BIT-WRAITH)";
            serde_json::json!({"content": [{"type": "text", "text": result}]})
        }
        _ => {
            serde_json::json!({"isError": true, "content": [{"type": "text",
                "text": format!("Tool '{}' is not implemented. No external action was performed.", name)}]})
        }
    }
}

fn process_request(req: JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: Some(list_tools()),
            error: None,
        },
        "tools/call" => {
            if let Some(params) = &req.params {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if !list_tools()["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|tool| tool["name"] == name)
                {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: req.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Unknown tool: {name}"),
                        }),
                    };
                }
                let args = params.get("arguments").unwrap_or(&serde_json::Value::Null);
                let result = handle_tool_call(name, args);
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(result),
                    error: None,
                }
            } else {
                JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "missing params".into(),
                    }),
                }
            }
        }
        "initialize" | "server/discover" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2026-07-28",
                "serverInfo": {"name": "bluesky-mcp-rs", "version": "0.1.0"},
                "capabilities": {"tools": {}}
            })),
            error: None,
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: req.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("unknown method: {}", req.method),
            }),
        },
    }
}

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut stdout = io::stdout();

    for line in reader.lines().flatten() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let resp = process_request(req);
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = writeln!(stdout, "{}", json);
                let _ = stdout.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn call(name: &str) -> JsonRpcResponse {
        process_request(JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(7)),
            method: "tools/call".into(),
            params: Some(serde_json::json!({"name": name, "arguments": {}})),
        })
    }
    #[test]
    fn unavailable_tools_report_errors() {
        for tool in [
            "bluesky_post",
            "bluesky_timeline",
            "bluesky_profile",
            "bluesky_search",
            "couchsky_profile",
            "couchsky_feed",
            "couchsky_search",
            "music_advance",
        ] {
            let response = call(tool);
            let result = response.result.unwrap();
            assert_eq!(result["isError"], true, "{tool}");
            assert!(result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("not implemented"));
        }
    }
    #[test]
    fn unknown_tool_is_protocol_error() {
        let response = call("not_a_tool");
        assert!(response.result.is_none());
        assert_eq!(response.error.unwrap().code, -32602);
    }
    #[test]
    fn music_results_are_explicit_samples() {
        for tool in ["music_now_playing", "music_choreographies"] {
            let result = call(tool).result.unwrap();
            assert!(result["content"][0]["text"]
                .as_str()
                .unwrap()
                .starts_with("Static sample"));
        }
    }
}
