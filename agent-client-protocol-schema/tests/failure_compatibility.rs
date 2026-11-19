//! Failure updates remain forward-compatible when local support is disabled.

#[cfg(feature = "unstable_protocol_v2")]
#[test]
fn failure_update_preserves_payload_with_or_without_local_support() {
    use agent_client_protocol_schema::v2::SessionUpdate;
    use serde_json::json;

    let wire = json!({
        "sessionUpdate": "failure",
        "failureId": "fail_1",
        "stopReason": "error",
        "error": {"code": -32603, "message": "Failed", "data": {"kind": "rate_limited"}},
    });
    let update: SessionUpdate = serde_json::from_value(wire.clone()).unwrap();
    #[cfg(feature = "unstable_failure_entries")]
    assert!(matches!(&update, SessionUpdate::Failure(_)));
    #[cfg(not(feature = "unstable_failure_entries"))]
    assert!(matches!(&update, SessionUpdate::Other(_)));
    assert_eq!(serde_json::to_value(update).unwrap(), wire);
}
