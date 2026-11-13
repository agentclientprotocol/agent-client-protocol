//! V2 receivers with and without local experimental features enabled.
//! Payload decoding does not negotiate capabilities or guarantee presentation.

#![cfg(feature = "unstable_protocol_v2")]

use agent_client_protocol_schema::v2::*;
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
        json!({"type": "file", "planId": "plan-1", "uri": "file:///workspace/plan.md"}),
    ] {
        let wire = json!({"sessionUpdate": "plan_update", "plan": plan});
        let update = decode_update(&wire);
        let SessionUpdate::PlanUpdate(plan_update) = &update else {
            panic!("expected plan update");
        };
        #[cfg(feature = "unstable_plan_operations")]
        assert!(matches!(
            plan_update.plan,
            PlanUpdateContent::Markdown(_) | PlanUpdateContent::File(_)
        ));
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
    for tag in ["items", "file", "markdown"] {
        assert_eq!(
            excluded
                .iter()
                .any(|variant| variant["properties"]["type"]["const"] == tag),
            tag == "items" || cfg!(feature = "unstable_plan_operations")
        );
    }
}
