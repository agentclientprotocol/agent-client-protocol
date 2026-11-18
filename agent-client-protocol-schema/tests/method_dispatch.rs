//! Aggregate parsing must select by method, not by overlapping payload shapes.

use agent_client_protocol_schema::MethodDispatchError;
use serde_json::{Value, json};

macro_rules! request_case {
    ($request:ident, $response:ident, $req:ident => $resp:ident, $method:expr, $params:expr, $result:expr) => {{
        let params = $params;
        let parsed = $request::from_method_and_params($method, &params).unwrap();
        assert_eq!(parsed.method(), $method);
        assert!($request::supports_method($method));
        let $request::$req(payload) = parsed else {
            panic!("wrong request variant for {}", $method);
        };
        let expected: $req = serde_json::from_value(params).unwrap();
        assert_eq!(
            serde_json::to_value(payload).unwrap(),
            serde_json::to_value(expected).unwrap()
        );

        let result = $result;
        let parsed = $response::from_method_and_result($method, result.clone()).unwrap();
        let $response::$resp(payload) = parsed else {
            panic!("wrong response variant for {}", $method);
        };
        let expected: $resp = serde_json::from_value(result).unwrap();
        assert_eq!(
            serde_json::to_value(payload).unwrap(),
            serde_json::to_value(expected).unwrap()
        );

        // Dispatch must preserve each individual type's deserialization rules.
        // In particular, do not normalize null, default bad input, or fall
        // through to the extension variant when a known method is malformed.
        for payload in [Value::Null, json!({}), json!(42), json!([])] {
            assert_eq!(
                $request::from_method_and_params($method, &payload).is_ok(),
                serde_json::from_value::<$req>(payload.clone()).is_ok(),
                "{} request: {payload}",
                $method,
            );
            assert_eq!(
                $response::from_method_and_result($method, payload.clone()).is_ok(),
                serde_json::from_value::<$resp>(payload.clone()).is_ok(),
                "{} response: {payload}",
                $method,
            );
        }
    }};
}

macro_rules! shared_tests {
    () => {
        #[test]
        fn session_requests_and_responses_keep_their_payloads() {
            request_case!(ClientRequest, AgentResponse, NewSessionRequest => NewSessionResponse,
                "session/new", json!({"cwd": "/work", "mcpServers": []}), json!({"sessionId": "s1"}));
            request_case!(ClientRequest, AgentResponse, ListSessionsRequest => ListSessionsResponse,
                "session/list", json!({}), json!({"sessions": []}));
            request_case!(ClientRequest, AgentResponse, DeleteSessionRequest => DeleteSessionResponse,
                "session/delete", json!({"sessionId": "s1"}), json!({}));
            request_case!(ClientRequest, AgentResponse, CloseSessionRequest => CloseSessionResponse,
                "session/close", json!({"sessionId": "s1"}), json!({}));
            request_case!(ClientRequest, AgentResponse, ResumeSessionRequest => ResumeSessionResponse,
                "session/resume", json!({"sessionId": "s1", "cwd": "/work"}), json!({}));
            request_case!(ClientRequest, AgentResponse, SetSessionConfigOptionRequest => SetSessionConfigOptionResponse,
                "session/set_config_option", json!({"sessionId": "s1", "configId": "mode", "type": "id", "value": "code"}),
                json!({"configOptions": []}));
            request_case!(AgentRequest, ClientResponse, CreateElicitationRequest => CreateElicitationResponse,
                "elicitation/create",
                json!({"sessionId": "s1", "mode": "form", "message": "Choose", "requestedSchema": {"type": "object", "properties": {}}}),
                json!({"action": "cancel"}));
        }

        #[test]
        fn unknown_methods_are_not_inferred_from_payloads() {
            let method = "session/unknown";
            macro_rules! unknown_params {
                ($enum:ident) => {
                    assert!(!$enum::supports_method(method));
                    assert!(matches!(
                        $enum::from_method_and_params(method, &json!({"sessionId": "s1"})),
                        Err(MethodDispatchError::UnknownMethod { method: name }) if name == method
                    ));
                };
            }
            unknown_params!(ClientRequest);
            unknown_params!(AgentRequest);
            unknown_params!(ClientNotification);
            unknown_params!(AgentNotification);
            unknown_params!(ProtocolLevelNotification);
            for error in [
                AgentResponse::from_method_and_result(method, json!({})).unwrap_err(),
                ClientResponse::from_method_and_result(method, json!({})).unwrap_err(),
            ] {
                assert!(matches!(
                    error, MethodDispatchError::UnknownMethod { method: name } if name == method
                ));
            }
        }

        #[test]
        fn known_malformed_payload_reports_its_method_and_source() {
            let error = ClientRequest::from_method_and_params("session/new", &json!({}))
                .unwrap_err();
            assert!(std::error::Error::source(&error).is_some());
            assert!(matches!(
                error, MethodDispatchError::InvalidPayload { method, .. } if method == "session/new"
            ));
            assert!(matches!(
                AgentResponse::from_method_and_result("session/new", json!({})),
                Err(MethodDispatchError::InvalidPayload { .. })
            ));
        }

        #[test]
        fn extensions_preserve_the_supplied_method_and_json_value() {
            let method = "_vendor/custom";
            for payload in [
                Value::Null,
                json!({}),
                json!({"method": "not-the-method", "params": {"nested": [1, null, true]}}),
                json!([1, 2]),
                json!("text"),
            ] {
                macro_rules! extension {
                    ($enum:ident, $variant:ident) => {
                        assert!($enum::supports_method(method));
                        let parsed = $enum::from_method_and_params(method, &payload).unwrap();
                        assert_eq!(parsed.method(), method);
                        assert_eq!(serde_json::to_value(&parsed).unwrap(), payload);
                        assert!(matches!(parsed, $enum::$variant(_)));
                    };
                }
                extension!(ClientRequest, ExtMethodRequest);
                extension!(AgentRequest, ExtMethodRequest);
                extension!(ClientNotification, ExtNotification);
                extension!(AgentNotification, ExtNotification);
                assert!(matches!(
                    AgentResponse::from_method_and_result(method, payload.clone()).unwrap(),
                    AgentResponse::ExtMethodResponse(ref response)
                        if serde_json::to_value(response).unwrap() == payload
                ));
                assert!(matches!(
                    ClientResponse::from_method_and_result(method, payload.clone()).unwrap(),
                    ClientResponse::ExtMethodResponse(ref response)
                        if serde_json::to_value(response).unwrap() == payload
                ));
            }
            assert!(!ProtocolLevelNotification::supports_method(method));
            assert!(matches!(
                ProtocolLevelNotification::from_method_and_params(method, &json!({})),
                Err(MethodDispatchError::UnknownMethod { .. })
            ));
        }

        #[test]
        fn protocol_notifications_require_their_method_and_payload() {
            for id in [json!(7), json!("request-7")] {
                let params = json!({"requestId": id});
                let parsed = ProtocolLevelNotification::from_method_and_params(
                    "$/cancel_request", &params,
                ).unwrap();
                assert_eq!(parsed.method(), "$/cancel_request");
                assert_eq!(serde_json::to_value(parsed).unwrap(), params);
            }
            assert!(ProtocolLevelNotification::supports_method("$/cancel_request"));
            assert!(!ProtocolLevelNotification::supports_method("$/unknown"));
            assert!(matches!(
                ProtocolLevelNotification::from_method_and_params("$/cancel_request", &json!({})),
                Err(MethodDispatchError::InvalidPayload { .. })
            ));
        }

        #[test]
        fn feature_gated_methods_follow_the_schema_features() {
            for (feature, method) in [
                (cfg!(feature = "unstable_llm_providers"), "providers/list"),
                (cfg!(feature = "unstable_llm_providers"), "providers/set"),
                (cfg!(feature = "unstable_llm_providers"), "providers/disable"),
                (cfg!(feature = "unstable_auth_status"), "auth/status"),
                (cfg!(feature = "unstable_session_fork"), "session/fork"),
                (cfg!(feature = "unstable_session_archive"), "session/archive"),
                (cfg!(feature = "unstable_session_archive"), "session/unarchive"),
                (cfg!(feature = "unstable_session_status"), "session/status"),
                (cfg!(feature = "unstable_session_options"), "session/options"),
                (cfg!(feature = "unstable_session_set_title"), "session/set_title"),
            ] {
                assert_eq!(ClientRequest::supports_method(method), feature, "{method}");
                for error in [
                    ClientRequest::from_method_and_params(method, &json!(42)).unwrap_err(),
                    AgentResponse::from_method_and_result(method, json!(42)).unwrap_err(),
                ] {
                    assert_eq!(
                        matches!(error, MethodDispatchError::InvalidPayload { .. }),
                        feature,
                        "{method}: {error}",
                    );
                }
            }
            let method = "mcp/message";
            let enabled = cfg!(feature = "unstable_mcp_over_acp");
            assert_eq!(AgentRequest::supports_method(method), enabled);
            assert_eq!(ClientNotification::supports_method(method), enabled);
            for error in [
                AgentRequest::from_method_and_params(method, &json!(42)).unwrap_err(),
                ClientResponse::from_method_and_result(method, json!(42)).unwrap_err(),
                ClientNotification::from_method_and_params(method, &json!(42)).unwrap_err(),
            ] {
                assert_eq!(
                    matches!(error, MethodDispatchError::InvalidPayload { .. }),
                    enabled,
                    "{error}",
                );
            }
        }
    };
}

mod v1 {
    use super::*;
    use agent_client_protocol_schema::v1::*;

    shared_tests!();

    #[test]
    fn v1_authentication_and_client_methods() {
        request_case!(ClientRequest, AgentResponse, InitializeRequest => InitializeResponse,
            "initialize", json!({"protocolVersion": 1}), json!({"protocolVersion": 1}));
        request_case!(ClientRequest, AgentResponse, AuthenticateRequest => AuthenticateResponse,
            "authenticate", json!({"methodId": "oauth"}), json!({}));
        request_case!(ClientRequest, AgentResponse, LogoutRequest => LogoutResponse,
            "logout", json!({}), json!({}));
        request_case!(ClientRequest, AgentResponse, LoadSessionRequest => LoadSessionResponse,
            "session/load", json!({"sessionId": "s1", "cwd": "/work", "mcpServers": []}), json!({}));
        request_case!(ClientRequest, AgentResponse, SetSessionModeRequest => SetSessionModeResponse,
            "session/set_mode", json!({"sessionId": "s1", "modeId": "code"}), json!({}));
        request_case!(ClientRequest, AgentResponse, PromptRequest => PromptResponse,
            "session/prompt", json!({"sessionId": "s1", "prompt": [{"type": "text", "text": "Hello"}]}),
            json!({"stopReason": "end_turn"}));
        request_case!(AgentRequest, ClientResponse, RequestPermissionRequest => RequestPermissionResponse,
            "session/request_permission",
            json!({"sessionId": "s1", "toolCall": {"toolCallId": "t1", "title": "Run tests"}, "options": []}),
            json!({"outcome": {"outcome": "cancelled"}}));
        request_case!(AgentRequest, ClientResponse, ReadTextFileRequest => ReadTextFileResponse,
            "fs/read_text_file", json!({"sessionId": "s1", "path": "/work/file"}), json!({"content": "hello"}));
        request_case!(AgentRequest, ClientResponse, WriteTextFileRequest => WriteTextFileResponse,
            "fs/write_text_file", json!({"sessionId": "s1", "path": "/work/file", "content": "hello"}), json!({}));
        request_case!(AgentRequest, ClientResponse, CreateTerminalRequest => CreateTerminalResponse,
            "terminal/create", json!({"sessionId": "s1", "command": "ls"}), json!({"terminalId": "t1"}));
        request_case!(AgentRequest, ClientResponse, TerminalOutputRequest => TerminalOutputResponse,
            "terminal/output", json!({"sessionId": "s1", "terminalId": "t1"}), json!({"output": "hello", "truncated": false}));
        request_case!(AgentRequest, ClientResponse, ReleaseTerminalRequest => ReleaseTerminalResponse,
            "terminal/release", json!({"sessionId": "s1", "terminalId": "t1"}), json!({}));
        request_case!(AgentRequest, ClientResponse, WaitForTerminalExitRequest => WaitForTerminalExitResponse,
            "terminal/wait_for_exit", json!({"sessionId": "s1", "terminalId": "t1"}), json!({"exitCode": 0}));
        request_case!(AgentRequest, ClientResponse, KillTerminalRequest => KillTerminalResponse,
            "terminal/kill", json!({"sessionId": "s1", "terminalId": "t1"}), json!({}));
    }

    #[test]
    fn v1_session_notification_selects_the_expected_variant() {
        let params = json!({
            "sessionId": "s1",
            "update": {"sessionUpdate": "tool_call_update", "toolCallId": "t1", "status": "in_progress"}
        });
        let parsed = AgentNotification::from_method_and_params("session/update", &params).unwrap();
        assert!(matches!(parsed, AgentNotification::SessionNotification(_)));
        assert_eq!(serde_json::to_value(parsed).unwrap(), params);
        let parsed = ClientNotification::from_method_and_params(
            "session/cancel",
            &json!({"sessionId": "s1"}),
        )
        .unwrap();
        assert!(matches!(parsed, ClientNotification::CancelNotification(_)));
    }
}

#[cfg(feature = "unstable_protocol_v2")]
mod v2 {
    use super::*;
    use agent_client_protocol_schema::v2::*;

    shared_tests!();

    #[test]
    fn v2_authentication_and_boxed_cancellation_payloads() {
        request_case!(ClientRequest, AgentResponse, InitializeRequest => InitializeResponse,
            "initialize", json!({"protocolVersion": 2, "info": {"name": "client", "version": "1"}}),
            json!({"protocolVersion": 2, "info": {"name": "agent", "version": "1"}}));
        request_case!(ClientRequest, AgentResponse, LoginAuthRequest => LoginAuthResponse,
            "auth/login", json!({"methodId": "oauth"}), json!({}));
        request_case!(ClientRequest, AgentResponse, LogoutAuthRequest => LogoutAuthResponse,
            "auth/logout", json!({}), json!({}));
        request_case!(ClientRequest, AgentResponse, CancelToolCallRequest => CancelToolCallResponse,
            "tool_call/cancel", json!({"sessionId": "s1", "toolCallId": "t1"}), json!({}));
        request_case!(ClientRequest, AgentResponse, CancelTerminalRequest => CancelTerminalResponse,
            "terminal/cancel", json!({"sessionId": "s1", "terminalId": "t1"}), json!({}));
        request_case!(ClientRequest, AgentResponse, PromptRequest => PromptResponse,
            "session/prompt", json!({"sessionId": "s1", "prompt": [{"type": "text", "text": "Hello"}]}),
            json!({"messageId": "m1", "stopReason": "end_turn"}));
        request_case!(AgentRequest, ClientResponse, RequestPermissionRequest => RequestPermissionResponse,
            "session/request_permission",
            json!({"sessionId": "s1", "title": "Run tests?", "subject": {"type": "tool_call", "toolCall": {"toolCallId": "t1", "title": "Run tests", "kind": "execute", "status": "pending"}}, "options": []}),
            json!({"outcome": {"outcome": "cancelled"}}));
    }

    #[test]
    fn v2_session_notification_selects_the_expected_variant() {
        let params = json!({
            "sessionId": "s1",
            "update": {"sessionUpdate": "tool_call_update", "toolCallId": "t1", "status": "in_progress"}
        });
        let parsed = AgentNotification::from_method_and_params("session/update", &params).unwrap();
        assert!(matches!(
            parsed,
            AgentNotification::UpdateSessionNotification(_)
        ));
        assert_eq!(serde_json::to_value(parsed).unwrap(), params);
        let parsed = ClientNotification::from_method_and_params(
            "session/cancel",
            &json!({"sessionId": "s1"}),
        )
        .unwrap();
        assert!(matches!(
            parsed,
            ClientNotification::CancelSessionNotification(_)
        ));
    }
}
