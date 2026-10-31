//! Tests for method-aware deserialization of the protocol enums.
//!
//! The enums' derived `Deserialize` implementations are `#[serde(untagged)]`
//! and exist for schema generation: serde tries variants in declaration
//! order, so a variant with all-optional fields silently matches any payload.
//! The `from_method_and_params` / `from_method_and_result` helpers dispatch
//! on the JSON-RPC method name instead and must route every documented
//! method to its intended variant.

use agent_client_protocol_schema::{MethodDispatchError, v1};
use serde_json::json;

#[test]
fn v1_session_new_routes_to_new_session_request() {
    // Regression test: with the derived untagged deserialization this exact
    // payload silently parsed as `ClientRequest::LogoutRequest`, dropping
    // `cwd` and `mcpServers` without any error.
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/new",
        Some(json!({
            "cwd": "/home/user/project",
            "mcpServers": [{"name": "filesystem", "command": "/path/to/mcp-server", "args": ["--stdio"], "env": []}],
        })),
    )
    .unwrap();
    match parsed {
        v1::ClientRequest::NewSessionRequest(req) => {
            assert_eq!(req.cwd, std::path::PathBuf::from("/home/user/project"));
            assert_eq!(req.mcp_servers.len(), 1);
        }
        other => panic!("expected NewSessionRequest, got {}", other.method()),
    }
}

#[test]
fn v1_stable_methods_route_to_their_variant() {
    // Client-to-agent methods.
    let client_cases: &[(&str, serde_json::Value)] = &[
        ("initialize", json!({"protocolVersion": 1})),
        ("authenticate", json!({"methodId": "oauth"})),
        ("logout", json!({})),
        ("session/new", json!({"cwd": "/tmp", "mcpServers": []})),
        (
            "session/load",
            json!({"sessionId": "s1", "cwd": "/tmp", "mcpServers": []}),
        ),
        ("session/list", json!({})),
        ("session/delete", json!({"sessionId": "s1"})),
        ("session/close", json!({"sessionId": "s1"})),
        ("session/resume", json!({"sessionId": "s1", "cwd": "/tmp"})),
        (
            "session/set_mode",
            json!({"sessionId": "s1", "modeId": "code"}),
        ),
        (
            "session/set_config_option",
            json!({"sessionId": "s1", "configId": "mode", "type": "id", "value": "code"}),
        ),
        (
            "session/prompt",
            json!({"sessionId": "s1", "prompt": [{"type": "text", "text": "hi"}]}),
        ),
    ];
    for (method, params) in client_cases {
        let parsed = v1::ClientRequest::from_method_and_params(method, Some(params.clone()))
            .unwrap_or_else(|e| panic!("{method} failed to parse: {e}"));
        assert_eq!(
            parsed.method(),
            *method,
            "{method} routed to the wrong variant",
        );
    }

    // Agent-to-client methods.
    let agent_cases: &[(&str, serde_json::Value)] = &[
        (
            "session/request_permission",
            json!({
                "sessionId": "s1",
                "toolCall": {"toolCallId": "t1", "title": "Run tests"},
                "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"}],
            }),
        ),
        (
            "fs/read_text_file",
            json!({"sessionId": "s1", "path": "/tmp/a"}),
        ),
        (
            "fs/write_text_file",
            json!({"sessionId": "s1", "path": "/tmp/a", "content": "x"}),
        ),
        (
            "terminal/create",
            json!({"sessionId": "s1", "command": "ls"}),
        ),
        (
            "terminal/output",
            json!({"sessionId": "s1", "terminalId": "t1"}),
        ),
        (
            "terminal/wait_for_exit",
            json!({"sessionId": "s1", "terminalId": "t1"}),
        ),
        (
            "terminal/kill",
            json!({"sessionId": "s1", "terminalId": "t1"}),
        ),
        (
            "terminal/release",
            json!({"sessionId": "s1", "terminalId": "t1"}),
        ),
        (
            "elicitation/create",
            json!({"sessionId": "s1", "mode": "form", "message": "pick one", "requestedSchema": {"type": "object", "properties": {}}}),
        ),
    ];
    for (method, params) in agent_cases {
        let parsed = v1::AgentRequest::from_method_and_params(method, Some(params.clone()))
            .unwrap_or_else(|e| panic!("{method} failed to parse: {e}"));
        assert_eq!(
            parsed.method(),
            *method,
            "{method} routed to the wrong variant",
        );
    }
}

#[test]
fn v1_extension_request_preserves_method_and_raw_params() {
    let parsed = v1::ClientRequest::from_method_and_params(
        "_com.example/workspace/buffers",
        Some(json!({"operation": "list"})),
    )
    .unwrap();
    match parsed {
        v1::ClientRequest::ExtMethodRequest(ext) => {
            assert_eq!(&*ext.method, "_com.example/workspace/buffers");
            let params: serde_json::Value = serde_json::from_str(ext.params.get()).unwrap();
            assert_eq!(params, json!({"operation": "list"}));
        }
        other => panic!("expected ExtMethodRequest, got {}", other.method()),
    }
}

#[test]
fn v1_unknown_method_is_an_error_not_a_silent_misroute() {
    let err =
        v1::ClientRequest::from_method_and_params("session/nope", Some(json!({}))).unwrap_err();
    assert!(
        matches!(err, MethodDispatchError::UnknownMethod { .. }),
        "{err}"
    );
}

#[test]
fn v1_mismatched_payload_is_an_error_not_a_silent_default() {
    // `session/new` requires `cwd`; the derived untagged deserialization used
    // to swallow this into an unrelated all-optional variant instead.
    let err =
        v1::ClientRequest::from_method_and_params("session/new", Some(json!({}))).unwrap_err();
    assert!(
        matches!(err, MethodDispatchError::InvalidPayload { .. }),
        "{err}"
    );
}

#[test]
fn v1_notifications_route_and_preserve_payload() {
    let parsed = v1::AgentNotification::from_method_and_params(
        "session/update",
        Some(json!({
            "sessionId": "s1",
            "update": {"sessionUpdate": "tool_call_update", "toolCallId": "t1", "status": "in_progress"},
        })),
    )
    .unwrap();
    match parsed {
        v1::AgentNotification::SessionNotification(notification) => {
            assert!(matches!(
                notification.update,
                v1::SessionUpdate::ToolCallUpdate(_)
            ));
        }
        other => panic!("expected SessionNotification, got {}", other.method()),
    }

    let parsed = v1::ClientNotification::from_method_and_params(
        "session/cancel",
        Some(json!({"sessionId": "s1"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v1::ClientNotification::CancelNotification(_)
    ));
}

#[test]
fn v1_responses_route_by_originating_method() {
    let parsed =
        v1::AgentResponse::from_method_and_result("session/new", Some(json!({"sessionId": "s1"})))
            .unwrap();
    match parsed {
        v1::AgentResponse::NewSessionResponse(response) => {
            assert_eq!(&*response.session_id.0, "s1");
        }
        other => panic!("expected NewSessionResponse, got {other:?}"),
    }

    let parsed = v1::ClientResponse::from_method_and_result(
        "fs/read_text_file",
        Some(json!({"content": "hello"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v1::ClientResponse::ReadTextFileResponse(_)
    ));
}

#[cfg(feature = "unstable_protocol_v2")]
#[test]
fn v2_requests_notifications_and_responses_route() {
    use agent_client_protocol_schema::v2;

    let parsed = v2::ClientRequest::from_method_and_params(
        "session/new",
        Some(json!({"cwd": "/home/user/project", "mcpServers": []})),
    )
    .unwrap();
    assert!(matches!(parsed, v2::ClientRequest::NewSessionRequest(_)));

    let parsed = v2::AgentNotification::from_method_and_params(
        "session/update",
        Some(json!({
            "sessionId": "s1",
            "update": {"sessionUpdate": "tool_call_update", "toolCallId": "t1", "status": "in_progress"},
        })),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v2::AgentNotification::UpdateSessionNotification(_)
    ));

    let parsed =
        v2::AgentResponse::from_method_and_result("session/new", Some(json!({"sessionId": "s1"})))
            .unwrap();
    assert!(matches!(parsed, v2::AgentResponse::NewSessionResponse(_)));

    let parsed =
        v2::ClientRequest::from_method_and_params("_com.example/thing", Some(json!({"x": 1})))
            .unwrap();
    assert!(matches!(parsed, v2::ClientRequest::ExtMethodRequest(_)));

    let err = v2::ClientRequest::from_method_and_params("session/nope", None).unwrap_err();
    assert!(
        matches!(err, MethodDispatchError::UnknownMethod { .. }),
        "{err}"
    );
}

#[test]
fn v1_protocol_level_cancel_request_routes() {
    let parsed = v1::ProtocolLevelNotification::from_method_and_params(
        "$/cancel_request",
        Some(json!({"requestId": 7})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v1::ProtocolLevelNotification::CancelRequestNotification(_)
    ));
    let err =
        v1::ProtocolLevelNotification::from_method_and_params("session/new", None).unwrap_err();
    assert!(
        matches!(err, MethodDispatchError::UnknownMethod { .. }),
        "{err}"
    );
}

#[test]
fn v1_extension_notification_and_response_preserve_raw_payload() {
    let parsed = v1::AgentNotification::from_method_and_params(
        "_com.example/progress",
        Some(json!({"percent": 42, "stage": "tests"})),
    )
    .unwrap();
    match parsed {
        v1::AgentNotification::ExtNotification(ext) => {
            assert_eq!(&*ext.method, "_com.example/progress");
            let params: serde_json::Value = serde_json::from_str(ext.params.get()).unwrap();
            assert_eq!(params, json!({"percent": 42, "stage": "tests"}));
        }
        other => panic!("expected ExtNotification, got {}", other.method()),
    }

    let parsed = v1::AgentResponse::from_method_and_result(
        "_com.example/query",
        Some(json!({"answer": [1, 2, 3]})),
    )
    .unwrap();
    match parsed {
        v1::AgentResponse::ExtMethodResponse(ext) => {
            let result: serde_json::Value = serde_json::from_str(ext.0.get()).unwrap();
            assert_eq!(result, json!({"answer": [1, 2, 3]}));
        }
        other => panic!("expected ExtMethodResponse, got {other:?}"),
    }
}

#[test]
fn v1_null_result_for_ack_style_response_is_accepted() {
    // Agents commonly answer ack-style requests with `result: null`; the
    // response structs for these methods have all-optional fields.
    let parsed = v1::AgentResponse::from_method_and_result("logout", None).unwrap();
    assert!(matches!(parsed, v1::AgentResponse::LogoutResponse(_)));
}

#[cfg(feature = "unstable_llm_providers")]
#[test]
fn v1_gated_provider_methods_route() {
    let parsed = v1::ClientRequest::from_method_and_params("providers/list", None).unwrap();
    assert!(matches!(parsed, v1::ClientRequest::ListProvidersRequest(_)));
    // Regression: before method-aware dispatch, providers/list was the
    // all-optional variant that silently swallowed session/new payloads.
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/new",
        Some(json!({"cwd": "/tmp", "mcpServers": []})),
    )
    .unwrap();
    assert!(matches!(parsed, v1::ClientRequest::NewSessionRequest(_)));
}

#[cfg(feature = "unstable_session_fork")]
#[test]
fn v1_gated_session_fork_routes() {
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/fork",
        Some(json!({"sessionId": "s1", "cwd": "/tmp", "mcpServers": []})),
    )
    .unwrap();
    assert!(matches!(parsed, v1::ClientRequest::ForkSessionRequest(_)));
}

#[cfg(feature = "unstable_session_archive")]
#[test]
fn v1_gated_session_archive_unarchive_route() {
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/archive",
        Some(json!({"sessionId": "s1"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v1::ClientRequest::ArchiveSessionRequest(_)
    ));
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/unarchive",
        Some(json!({"sessionId": "s1"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v1::ClientRequest::UnarchiveSessionRequest(_)
    ));
}

#[cfg(feature = "unstable_session_status")]
#[test]
fn v1_gated_session_status_routes() {
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/status",
        Some(json!({"sessionId": "s1"})),
    )
    .unwrap();
    assert!(matches!(parsed, v1::ClientRequest::SessionStatusRequest(_)));
}

#[cfg(feature = "unstable_session_set_title")]
#[test]
fn v1_gated_session_set_title_routes() {
    let parsed = v1::ClientRequest::from_method_and_params(
        "session/set_title",
        Some(json!({"sessionId": "s1", "title": "Fix the bug"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v1::ClientRequest::SetSessionTitleRequest(_)
    ));
}

#[cfg(feature = "unstable_auth_status")]
#[test]
fn v1_gated_auth_status_routes() {
    let parsed = v1::ClientRequest::from_method_and_params("auth/status", None).unwrap();
    assert!(matches!(parsed, v1::ClientRequest::AuthStatusRequest(_)));
}

#[cfg(feature = "unstable_mcp_over_acp")]
#[test]
fn v1_gated_mcp_message_routes_both_directions() {
    let params = json!({
        "serverId": "srv1",
        "requestId": "req1",
        "method": "tools/list",
        "params": null
    });
    // `mcp/message` is an agent-to-client request in v1.
    let parsed = v1::AgentRequest::from_method_and_params("mcp/message", Some(params)).unwrap();
    assert!(matches!(parsed, v1::AgentRequest::MessageMcpRequest(_)));
}

#[cfg(feature = "unstable_protocol_v2")]
#[test]
fn v2_gated_and_boxed_variants_route() {
    use agent_client_protocol_schema::v2;

    let parsed = v2::ClientRequest::from_method_and_params(
        "tool_call/cancel",
        Some(json!({"sessionId": "s1", "toolCallId": "t1"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v2::ClientRequest::CancelToolCallRequest(_)
    ));

    let parsed = v2::ClientRequest::from_method_and_params(
        "terminal/cancel",
        Some(json!({"sessionId": "s1", "terminalId": "t1"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v2::ClientRequest::CancelTerminalRequest(_)
    ));

    let parsed = v2::ClientNotification::from_method_and_params(
        "session/cancel",
        Some(json!({"sessionId": "s1"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v2::ClientNotification::CancelSessionNotification(_)
    ));

    // Boxed response variants keep their payload.
    let parsed = v2::ClientResponse::from_method_and_result(
        "session/request_permission",
        Some(json!({"outcome": {"outcome": "selected", "optionId": "allow"}})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v2::ClientResponse::RequestPermissionResponse(_)
    ));
}

#[cfg(feature = "unstable_protocol_v2")]
#[test]
fn v2_stable_methods_route_to_their_variant() {
    use agent_client_protocol_schema::v2;

    // Client-to-agent methods.
    let client_cases: &[(&str, serde_json::Value)] = &[
        (
            "initialize",
            json!({"protocolVersion": 2, "info": {"name": "c", "version": "1"}}),
        ),
        ("auth/login", json!({"methodId": "oauth"})),
        ("auth/logout", json!({})),
        ("session/new", json!({"cwd": "/tmp", "mcpServers": []})),
        ("session/list", json!({})),
        ("session/delete", json!({"sessionId": "s1"})),
        ("session/close", json!({"sessionId": "s1"})),
        ("session/resume", json!({"sessionId": "s1", "cwd": "/tmp"})),
        (
            "session/set_config_option",
            json!({"sessionId": "s1", "configId": "mode", "type": "id", "value": "code"}),
        ),
        (
            "session/prompt",
            json!({"sessionId": "s1", "prompt": [{"type": "text", "text": "hi"}]}),
        ),
    ];
    for (method, params) in client_cases {
        let parsed = v2::ClientRequest::from_method_and_params(method, Some(params.clone()))
            .unwrap_or_else(|e| panic!("{method} failed to parse: {e}"));
        assert_eq!(
            parsed.method(),
            *method,
            "{method} routed to the wrong variant"
        );
    }

    // Agent-to-client methods.
    let agent_cases: &[(&str, serde_json::Value)] = &[
        (
            "session/request_permission",
            json!({
                "sessionId": "s1",
                "title": "Run this script?",
                "subject": {"type": "tool_call", "toolCall": {"toolCallId": "t1", "title": "Execute setup script", "kind": "execute", "status": "pending"}},
                "options": [{"optionId": "allow", "name": "Allow", "kind": "allow_once"}],
            }),
        ),
        (
            "elicitation/create",
            json!({"sessionId": "s1", "mode": "form", "message": "pick one", "requestedSchema": {"type": "object", "properties": {}}}),
        ),
    ];
    for (method, params) in agent_cases {
        let parsed = v2::AgentRequest::from_method_and_params(method, Some(params.clone()))
            .unwrap_or_else(|e| panic!("{method} failed to parse: {e}"));
        assert_eq!(
            parsed.method(),
            *method,
            "{method} routed to the wrong variant"
        );
    }

    // Notifications.
    let parsed = v2::AgentNotification::from_method_and_params(
        "session/update",
        Some(json!({"sessionId": "s1", "update": {"sessionUpdate": "plan_update", "plan": {"type": "items", "planId": "p1", "entries": []}}})),
    )
    .unwrap();
    assert_eq!(parsed.method(), "session/update");
    let parsed = v2::ClientNotification::from_method_and_params(
        "session/cancel",
        Some(json!({"sessionId": "s1"})),
    )
    .unwrap();
    assert_eq!(parsed.method(), "session/cancel");

    // Responses route by the originating method.
    let parsed = v2::AgentResponse::from_method_and_result(
        "session/prompt",
        Some(json!({"messageId": "m1", "stopReason": "end_turn"})),
    )
    .unwrap();
    assert!(matches!(parsed, v2::AgentResponse::PromptResponse(_)));
    let parsed = v2::ClientResponse::from_method_and_result(
        "elicitation/create",
        Some(json!({"action": "cancel"})),
    )
    .unwrap();
    assert!(matches!(
        parsed,
        v2::ClientResponse::CreateElicitationResponse(_)
    ));
}
