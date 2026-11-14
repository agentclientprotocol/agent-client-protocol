//! V2 receivers with and without local experimental features enabled.
//! Payload decoding does not negotiate capabilities or guarantee presentation.

#![cfg(feature = "unstable_protocol_v2")]

use agent_client_protocol_schema::{MaybeUndefined, v2::*};
use serde_json::{Value, json};

fn decode_update(wire: &Value) -> SessionUpdate {
    let update = serde_json::from_value::<SessionUpdate>(wire.clone()).unwrap();
    assert_eq!(
        serde_json::from_str::<SessionUpdate>(&wire.to_string()).unwrap(),
        update
    );
    update
}

#[test]
fn experimental_plan_content_roundtrips_with_or_without_local_support() {
    for plan in [
        json!({"type": "markdown", "planId": "plan-1", "content": "# Next steps"}),
        json!({
            "type": "markdown",
            "planId": "plan-1",
            "content": "# Next steps",
            "path": "/workspace/plan.md"
        }),
    ] {
        let wire = json!({"sessionUpdate": "plan_update", "plan": plan});
        let update = decode_update(&wire);
        let SessionUpdate::PlanUpdate(plan_update) = &update else {
            panic!("expected plan update");
        };
        #[cfg(feature = "unstable_plan_operations")]
        assert!(matches!(plan_update.plan, PlanUpdateContent::Markdown(_)));
        #[cfg(not(feature = "unstable_plan_operations"))]
        assert!(matches!(plan_update.plan, PlanUpdateContent::Other(_)));
        assert_eq!(serde_json::to_value(update).unwrap(), wire);
    }
}

#[test]
fn experimental_plan_removal_roundtrips_with_or_without_local_support() {
    let wire = json!({"sessionUpdate": "plan_removed", "planId": "plan-1"});
    let update = decode_update(&wire);
    #[cfg(feature = "unstable_plan_operations")]
    assert!(matches!(update, SessionUpdate::PlanRemoved(_)));
    #[cfg(not(feature = "unstable_plan_operations"))]
    assert!(matches!(update, SessionUpdate::Other(_)));
    assert_eq!(serde_json::to_value(update).unwrap(), wire);
}

#[test]
fn experimental_prompt_suggestions_roundtrip_with_or_without_local_support() {
    for suggestions in [
        json!([]),
        json!([{"prompt": [{"type": "text", "text": "Run the tests"}]}]),
    ] {
        let wire = json!({
            "sessionUpdate": "prompt_suggestions_update",
            "suggestions": suggestions
        });
        let update = decode_update(&wire);
        #[cfg(feature = "unstable_prompt_suggestions")]
        assert!(matches!(update, SessionUpdate::PromptSuggestionsUpdate(_)));
        #[cfg(not(feature = "unstable_prompt_suggestions"))]
        assert!(matches!(update, SessionUpdate::Other(_)));
        assert_eq!(serde_json::to_value(update).unwrap(), wire);
    }
}

#[test]
fn experimental_skill_commands_roundtrip_with_or_without_local_support() {
    let wire = json!({
        "sessionUpdate": "available_commands_update",
        "availableCommands": [
            {
                "type": "skill",
                "name": "pdf-processing",
                "description": "Extract text and tables from PDF files",
                "path": "/home/alice/.agents/skills/pdf-processing/SKILL.md"
            },
            {"type": "command", "name": "test", "description": "Run project tests"}
        ]
    });
    let update = decode_update(&wire);
    let SessionUpdate::AvailableCommandsUpdate(commands) = &update else {
        panic!("expected available commands update");
    };
    let [skill, command] = commands.available_commands.as_slice() else {
        panic!("expected both entries to be kept");
    };
    #[cfg(feature = "unstable_agent_skills")]
    assert!(matches!(skill, AvailableCommand::Skill(_)));
    #[cfg(not(feature = "unstable_agent_skills"))]
    assert!(matches!(skill, AvailableCommand::Other(_)));
    assert_eq!(skill.name(), "pdf-processing");
    assert!(matches!(command, AvailableCommand::Command(_)));
    assert_eq!(serde_json::to_value(update).unwrap(), wire);
}

#[test]
fn experimental_skill_references_roundtrip_with_or_without_local_support() {
    let wire = json!({
        "sessionUpdate": "user_message",
        "messageId": "msg_user_1",
        "content": [
            {"type": "skill", "name": "pdf-processing"},
            {"type": "text", "text": "Summarize report.pdf"}
        ]
    });
    let update = decode_update(&wire);
    let SessionUpdate::UserMessage(message) = &update else {
        panic!("expected user message");
    };
    let MaybeUndefined::Value(content) = &message.content else {
        panic!("expected content");
    };
    let [skill, _text] = content.as_slice() else {
        panic!("expected both blocks to be kept");
    };
    #[cfg(feature = "unstable_agent_skills")]
    assert!(matches!(skill, ContentBlock::Skill(_)));
    #[cfg(not(feature = "unstable_agent_skills"))]
    assert!(matches!(skill, ContentBlock::Other(_)));
    assert_eq!(serde_json::to_value(update).unwrap(), wire);
}

#[test]
fn unknown_standard_and_custom_updates_preserve_their_payloads() {
    for tag in ["future_display_update", "_acme_display_update"] {
        let wire = json!({
            "sessionUpdate": tag,
            "data": {"items": [1, null, "text"]},
            "_meta": {"acme.dev": {"revision": 2}}
        });
        let update = decode_update(&wire);
        assert!(matches!(update, SessionUpdate::Other(_)));
        assert_eq!(serde_json::to_value(update).unwrap(), wire);
    }
}

#[test]
fn optional_experimental_usage_does_not_block_idle_transition() {
    for usage in [
        None,
        Some(Value::Null),
        Some(json!({"totalTokens": 30, "inputTokens": 20, "outputTokens": 10})),
        Some(json!({"totalTokens": "invalid"})),
    ] {
        let mut wire = json!({
            "sessionUpdate": "state_update",
            "state": "idle",
            "stopReason": "end_turn"
        });
        if let Some(usage) = &usage {
            wire["usage"] = usage.clone();
        }
        let update = decode_update(&wire);
        let SessionUpdate::StateUpdate(StateUpdate::Idle(idle)) = update else {
            panic!("expected idle state");
        };
        assert_eq!(idle.stop_reason, Some(StopReason::EndTurn));
        #[cfg(feature = "unstable_end_turn_token_usage")]
        {
            let expected = usage.filter(|value| value["totalTokens"].is_u64());
            assert_eq!(
                idle.usage.map(|usage| serde_json::to_value(usage).unwrap()),
                expected
            );
        }
    }
}

#[cfg(feature = "unstable_llm_providers")]
#[test]
fn experimental_types_do_not_imply_provider_capability() {
    for wire in [json!({}), json!({"providers": null})] {
        let capabilities: AgentCapabilities = serde_json::from_value(wire).unwrap();
        assert!(capabilities.providers.is_none());
    }
    let capabilities: AgentCapabilities = serde_json::from_value(json!({"providers": {}})).unwrap();
    assert!(capabilities.providers.is_some());
}

#[cfg(feature = "schemars")]
#[test]
fn plan_fallback_schema_excludes_only_locally_recognized_variants() {
    let schema = serde_json::to_value(schemars::schema_for!(OtherPlanUpdateContent)).unwrap();
    let excluded = schema["not"]["anyOf"].as_array().unwrap();
    for tag in ["items", "markdown"] {
        assert_eq!(
            excluded
                .iter()
                .any(|variant| variant["properties"]["type"]["const"] == tag),
            tag == "items" || cfg!(feature = "unstable_plan_operations")
        );
    }
}

#[cfg(feature = "schemars")]
#[test]
fn session_update_fallback_schema_excludes_only_locally_recognized_variants() {
    let schema = serde_json::to_value(schemars::schema_for!(OtherSessionUpdate)).unwrap();
    let excluded = schema["not"]["anyOf"].as_array().unwrap();
    for (tag, recognized) in [
        ("plan_update", true),
        ("plan_removed", cfg!(feature = "unstable_plan_operations")),
        (
            "prompt_suggestions_update",
            cfg!(feature = "unstable_prompt_suggestions"),
        ),
    ] {
        assert_eq!(
            excluded
                .iter()
                .any(|variant| variant["properties"]["sessionUpdate"]["const"] == tag),
            recognized
        );
    }
}

#[cfg(feature = "schemars")]
#[test]
fn skill_fallback_schemas_exclude_only_locally_recognized_variants() {
    for schema in [
        serde_json::to_value(schemars::schema_for!(OtherAvailableCommand)).unwrap(),
        serde_json::to_value(schemars::schema_for!(OtherContentBlock)).unwrap(),
    ] {
        let excluded = schema["not"]["anyOf"].as_array().unwrap();
        assert_eq!(
            excluded
                .iter()
                .any(|variant| variant["properties"]["type"]["const"] == "skill"),
            cfg!(feature = "unstable_agent_skills")
        );
    }
}
