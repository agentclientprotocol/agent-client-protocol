//! Wire-format regression tests shared by the two protocol versions.
//! These do not materialize session history or enforce lifecycle rules.

macro_rules! compaction_tests {
    ($version:ident) => {
        mod $version {
            use agent_client_protocol_schema::$version::*;
            use serde_json::{Value, json};

            #[test]
            fn compaction_statuses_roundtrip() {
                assert_eq!(CompactionStatus::default(), CompactionStatus::InProgress);

                for (wire, status) in [
                    ("in_progress", CompactionStatus::InProgress),
                    ("completed", CompactionStatus::Completed),
                    ("failed", CompactionStatus::Failed),
                    ("cancelled", CompactionStatus::Cancelled),
                    ("paused", CompactionStatus::Other("paused".into())),
                    ("_acme_paused", CompactionStatus::Other("_acme_paused".into())),
                ] {
                    assert_eq!(serde_json::to_value(&status).unwrap(), json!(wire));
                    assert_eq!(
                        serde_json::from_value::<CompactionStatus>(json!(wire)).unwrap(),
                        status
                    );
                    let update = SessionUpdate::CompactionUpdate(CompactionUpdate::new(
                        CompactionId::new("cmp_001"),
                        status,
                    ));
                    let wire_update = json!({
                        "sessionUpdate": "compaction_update",
                        "compactionId": "cmp_001",
                        "status": wire
                    });
                    assert_eq!(serde_json::to_value(&update).unwrap(), wire_update);
                    assert_eq!(
                        serde_json::from_value::<SessionUpdate>(wire_update.clone()).unwrap(),
                        update
                    );

                    let mut with_error = wire_update;
                    with_error["error"] = json!("compaction error details");
                    let parsed: SessionUpdate =
                        serde_json::from_value(with_error.clone()).unwrap();
                    assert_eq!(serde_json::to_value(parsed).unwrap(), with_error);
                }
            }

            #[test]
            fn compaction_patch_fields_roundtrip_independently() {
                // Omitted/null/empty/non-empty summaries combine independently
                // with omitted/null/concrete error and metadata patches.
                for summary in [
                    None,
                    Some(Value::Null),
                    Some(json!([])),
                    Some(json!([{"type": "text", "text": "partial summary"}])),
                ] {
                    for error in [None, Some(Value::Null), Some(json!("context limit"))] {
                        for meta in [None, Some(Value::Null), Some(json!({"attempt": 2}))] {
                            let mut wire = json!({
                                "sessionUpdate": "compaction_update",
                                "compactionId": "cmp_001",
                                "status": "failed"
                            });
                            for (key, value) in [
                                ("summary", &summary),
                                ("error", &error),
                                ("_meta", &meta),
                            ] {
                                if let Some(value) = value {
                                    wire[key] = value.clone();
                                }
                            }
                            let parsed: SessionUpdate =
                                serde_json::from_value(wire.clone()).unwrap();
                            let SessionUpdate::CompactionUpdate(update) = &parsed else {
                                panic!("expected typed compaction update");
                            };
                            assert_eq!(update.summary.is_undefined(), summary.is_none());
                            assert_eq!(update.summary.is_null(), summary == Some(Value::Null));
                            assert_eq!(
                                update.summary.value().is_some(),
                                summary.as_ref().is_some_and(|value| !value.is_null())
                            );
                            assert_eq!(update.error.is_undefined(), error.is_none());
                            assert_eq!(update.error.is_null(), error == Some(Value::Null));
                            assert_eq!(
                                update.error.value().map(String::as_str),
                                error.as_ref().and_then(Value::as_str)
                            );
                            assert_eq!(update.meta.is_undefined(), meta.is_none());
                            assert_eq!(update.meta.is_null(), meta == Some(Value::Null));
                            assert_eq!(
                                update.meta.value().map(|value| json!(value)),
                                meta.clone().filter(|value| !value.is_null())
                            );
                            assert_eq!(serde_json::to_value(parsed).unwrap(), wire);
                        }
                    }
                }

                let completed = SessionUpdate::CompactionUpdate(
                    CompactionUpdate::new("cmp_001", CompactionStatus::Completed)
                        .summary(vec![ContentBlock::Text(TextContent::new("retained"))]),
                );
                let wire = json!({
                    "sessionUpdate": "compaction_update",
                    "compactionId": "cmp_001",
                    "status": "completed",
                    "summary": [{"type": "text", "text": "retained"}]
                });
                assert_eq!(serde_json::to_value(&completed).unwrap(), wire);
                assert_eq!(serde_json::from_value::<SessionUpdate>(wire).unwrap(), completed);

                // Omission remains omission on terminal updates, not an implicit clear.
                for status in [CompactionStatus::Failed, CompactionStatus::Cancelled] {
                    let update = CompactionUpdate::new("cmp_001", status);
                    assert!(update.summary.is_undefined());
                    assert!(serde_json::to_value(update).unwrap().get("summary").is_none());
                }
            }

            #[test]
            fn compaction_replay_summary_patches_preserve_replace_or_clear_content() {
                for status in ["in_progress", "completed", "failed", "cancelled", "paused"] {
                    for summary in [
                        None,
                        Some(Value::Null),
                        Some(json!([])),
                        Some(json!([{"type": "text", "text": "snapshot summary"}])),
                    ] {
                        let mut wire = json!({
                            "sessionUpdate": "compaction_update",
                            "compactionId": "cmp_001",
                            "status": status
                        });
                        if let Some(summary) = &summary {
                            wire["summary"] = summary.clone();
                        }
                        let SessionUpdate::CompactionUpdate(update) =
                            serde_json::from_value(wire.clone()).unwrap()
                        else {
                            panic!("expected materialized compaction update");
                        };
                        assert_eq!(
                            serde_json::to_value(SessionUpdate::CompactionUpdate(update.clone()))
                                .unwrap(),
                            wire
                        );

                        // Exercise the existing patch helper, not a session
                        // materializer. Summary patches do not depend on status.
                        let existing = vec![ContentBlock::Text(TextContent::new("partial"))];
                        let expected: Option<Vec<ContentBlock>> = summary
                            .as_ref()
                            .filter(|value| !value.is_null())
                            .map(|value| serde_json::from_value(value.clone()).unwrap());
                        let mut cached = Some(existing.clone());
                        let mut fresh = None;
                        update.summary.clone().update_to(&mut cached);
                        update.summary.update_to(&mut fresh);
                        assert_eq!(fresh, expected);
                        if summary.is_none() {
                            assert_eq!(cached, Some(existing));
                        } else {
                            assert_eq!(cached, fresh);
                        }
                    }
                }
            }

            #[test]
            fn compaction_chunks_after_terminal_snapshots_append_and_replace_in_receive_order() {
                for status in [
                    CompactionStatus::Completed,
                    CompactionStatus::Failed,
                    CompactionStatus::Cancelled,
                ] {
                    let terminal = SessionUpdate::CompactionUpdate(
                        CompactionUpdate::new("cmp_001", status.clone()),
                    );
                    let SessionUpdate::CompactionUpdate(snapshot) =
                        serde_json::from_value(serde_json::to_value(&terminal).unwrap()).unwrap()
                    else {
                        panic!("expected status snapshot");
                    };
                    assert_eq!(snapshot.status, status);

                    // Exercise wire round trips and content patches, not a full
                    // Client reducer. Status does not seal summary delivery.
                    let mut summary = None;
                    snapshot.summary.update_to(&mut summary);
                    for text in ["late first", "late second"] {
                        let chunk = SessionUpdate::CompactionSummaryChunk(
                            CompactionSummaryChunk::new(
                                "cmp_001",
                                ContentBlock::Text(TextContent::new(text)),
                            ),
                        );
                        let SessionUpdate::CompactionSummaryChunk(parsed) =
                            serde_json::from_value(serde_json::to_value(&chunk).unwrap()).unwrap()
                        else {
                            panic!("expected late summary chunk");
                        };
                        assert_eq!(parsed.compaction_id, CompactionId::new("cmp_001"));
                        summary.get_or_insert_with(Vec::new).push(parsed.content);
                    }
                    assert_eq!(
                        summary.as_ref().unwrap().as_slice(),
                        &[
                            ContentBlock::Text(TextContent::new("late first")),
                            ContentBlock::Text(TextContent::new("late second")),
                        ]
                    );

                    let replacement = vec![ContentBlock::Text(TextContent::new("replacement"))];
                    let patch = SessionUpdate::CompactionUpdate(
                        CompactionUpdate::new("cmp_001", status.clone())
                            .summary(replacement.clone()),
                    );
                    let SessionUpdate::CompactionUpdate(parsed) =
                        serde_json::from_value(serde_json::to_value(&patch).unwrap()).unwrap()
                    else {
                        panic!("expected same-status summary patch");
                    };
                    assert_eq!(parsed.status, status);
                    parsed.summary.update_to(&mut summary);
                    assert_eq!(summary, Some(replacement.clone()));

                    let chunk = SessionUpdate::CompactionSummaryChunk(
                        CompactionSummaryChunk::new(
                            "cmp_001",
                            ContentBlock::Text(TextContent::new("after replacement")),
                        ),
                    );
                    let SessionUpdate::CompactionSummaryChunk(parsed) =
                        serde_json::from_value(serde_json::to_value(&chunk).unwrap()).unwrap()
                    else {
                        panic!("expected post-replacement chunk");
                    };
                    summary.as_mut().unwrap().push(parsed.content.clone());
                    let mut expected = replacement;
                    expected.push(parsed.content);
                    assert_eq!(summary, Some(expected));
                }
            }

            #[test]
            fn compaction_chunk_first_uses_client_defaults() {
                let SessionUpdate::CompactionSummaryChunk(chunk) =
                    serde_json::from_value(json!({
                        "sessionUpdate": "compaction_summary_chunk",
                        "compactionId": "cmp_001",
                        "content": {"type": "text", "text": "streamed first"},
                        "_meta": {"chunk": 1}
                    })).unwrap()
                else {
                    panic!("expected first-seen summary chunk");
                };

                // Use schema constructors and patch helpers to exercise Client
                // defaults; this is not a full session materializer.
                let initial = CompactionUpdate::new(
                    chunk.compaction_id.clone(),
                    CompactionStatus::default(),
                );
                assert_eq!(initial.status, CompactionStatus::InProgress);
                assert!(initial.summary.is_undefined());
                assert!(initial.error.is_undefined());
                assert!(initial.meta.is_undefined());
                let mut summary = Some(vec![chunk.content]);

                let SessionUpdate::CompactionUpdate(completed) =
                    serde_json::from_value(json!({
                        "sessionUpdate": "compaction_update",
                        "compactionId": "cmp_001",
                        "status": "completed"
                    })).unwrap()
                else {
                    panic!("expected later compaction update");
                };
                assert_eq!(completed.compaction_id, initial.compaction_id);
                assert_eq!(completed.status, CompactionStatus::Completed);
                completed.summary.update_to(&mut summary);
                assert_eq!(
                    summary,
                    Some(vec![ContentBlock::Text(TextContent::new("streamed first"))])
                );
            }

            #[test]
            fn compaction_summary_chunk_metadata_roundtrip() {
                for meta in [None, Some(Value::Null), Some(json!({"chunk": 1}))] {
                    let mut wire = json!({
                        "sessionUpdate": "compaction_summary_chunk",
                        "compactionId": "cmp_001",
                        "content": {"type": "text", "text": "provisional", "_meta": {"source": "agent"}}
                    });
                    if let Some(meta) = &meta {
                        wire["_meta"] = meta.clone();
                    }
                    let parsed: SessionUpdate = serde_json::from_value(wire.clone()).unwrap();
                    let SessionUpdate::CompactionSummaryChunk(chunk) = &parsed else {
                        panic!("expected typed summary chunk");
                    };
                    assert_eq!(
                        chunk.meta.as_ref().map(|value| json!(value)),
                        meta.filter(|value| !value.is_null())
                    );
                    // Unlike update patches, null chunk metadata means absent.
                    if wire.get("_meta").is_some_and(Value::is_null) {
                        wire.as_object_mut().unwrap().remove("_meta");
                    }
                    assert_eq!(serde_json::to_value(parsed).unwrap(), wire);
                }

                let chunk = CompactionSummaryChunk::new(
                    "cmp_001",
                    ContentBlock::Text(TextContent::new("provisional")),
                )
                .meta(serde_json::from_value::<Meta>(json!({"chunk": 1})).unwrap());
                assert_eq!(serde_json::to_value(chunk).unwrap()["_meta"], json!({"chunk": 1}));
            }

            #[test]
            fn compaction_patch_builders_preserve_three_states() {
                use agent_client_protocol_schema::MaybeUndefined;

                let nulls = CompactionUpdate::new("cmp_001", CompactionStatus::Failed)
                    .summary(None)
                    .error(None)
                    .meta(None);
                assert_eq!(
                    serde_json::to_value(&nulls).unwrap(),
                    json!({
                        "compactionId": "cmp_001", "status": "failed",
                        "summary": null, "error": null, "_meta": null
                    })
                );
                let omitted = nulls
                    .summary(MaybeUndefined::Undefined)
                    .error(MaybeUndefined::Undefined)
                    .meta(MaybeUndefined::Undefined);
                assert_eq!(
                    omitted,
                    CompactionUpdate::new("cmp_001", CompactionStatus::Failed)
                );
                let concrete = omitted
                    .summary(Vec::new())
                    .error(String::from("context limit"))
                    .meta(serde_json::from_value::<Meta>(json!({"attempt": 2})).unwrap());
                assert_eq!(
                    serde_json::to_value(concrete).unwrap(),
                    json!({
                        "compactionId": "cmp_001", "status": "failed",
                        "summary": [], "error": "context limit", "_meta": {"attempt": 2}
                    })
                );
            }

            #[test]
            fn compaction_required_fields_reject_missing_null_and_malformed_values() {
                for (wire, fields) in [
                    (
                        json!({
                            "sessionUpdate": "compaction_update",
                            "compactionId": "cmp_001",
                            "status": "completed"
                        }),
                        vec![
                            ("sessionUpdate", json!(42)),
                            ("compactionId", json!(42)),
                            ("status", json!({})),
                        ],
                    ),
                    (
                        json!({
                            "sessionUpdate": "compaction_summary_chunk",
                            "compactionId": "cmp_001",
                            "content": {"type": "text", "text": "provisional"}
                        }),
                        vec![
                            ("sessionUpdate", json!(42)),
                            ("compactionId", json!(42)),
                            ("content", json!({"type": "text"})),
                        ],
                    ),
                ] {
                    for (field, malformed) in fields {
                        for replacement in [None, Some(Value::Null), Some(malformed)] {
                            let mut invalid = wire.clone();
                            if let Some(value) = replacement {
                                invalid[field] = value;
                            } else {
                                invalid.as_object_mut().unwrap().remove(field);
                            }
                            assert!(
                                serde_json::from_value::<SessionUpdate>(invalid.clone()).is_err(),
                                "known discriminator must not hide invalid {field}: {invalid}"
                            );
                            // The discriminator belongs to the union, not the
                            // standalone payload types.
                            if field == "sessionUpdate" {
                                continue;
                            }
                            let invalid_typed = if wire["sessionUpdate"] == "compaction_update" {
                                serde_json::from_value::<CompactionUpdate>(invalid).is_err()
                            } else {
                                serde_json::from_value::<CompactionSummaryChunk>(invalid).is_err()
                            };
                            assert!(invalid_typed, "required {field} must be rejected");
                        }
                    }
                }
            }

            #[test]
            fn compaction_optional_fields_follow_existing_deserialization_tolerance() {
                let update: CompactionUpdate = serde_json::from_value(json!({
                    "compactionId": "cmp_001",
                    "status": "failed",
                    "summary": false,
                    "error": 42,
                    "_meta": "invalid"
                }))
                .unwrap();
                assert!(update.summary.is_undefined());
                assert!(update.error.is_undefined());
                assert!(update.meta.is_undefined());

                let update: CompactionUpdate = serde_json::from_value(json!({
                    "compactionId": "cmp_001",
                    "status": "completed",
                    "summary": [
                        {"type": "text", "text": "retained"},
                        {"type": "text"}
                    ]
                }))
                .unwrap();
                assert_eq!(
                    update.summary.value().unwrap(),
                    &[ContentBlock::Text(TextContent::new("retained"))]
                );
                let chunk: CompactionSummaryChunk = serde_json::from_value(json!({
                    "compactionId": "cmp_001",
                    "content": {"type": "text", "text": "provisional"},
                    "_meta": false
                }))
                .unwrap();
                assert!(chunk.meta.is_none());
            }
        }
    };
}

compaction_tests!(v1);
#[cfg(feature = "unstable_protocol_v2")]
compaction_tests!(v2);

#[test]
fn v1_compaction_capability_gate_roundtrip() {
    use agent_client_protocol_schema::v1::*;
    use serde_json::json;

    for wire in [json!({}), json!({"session": null})] {
        let capabilities: ClientCapabilities = serde_json::from_value(wire).unwrap();
        assert!(capabilities.session.is_none());
        assert!(
            serde_json::to_value(capabilities)
                .unwrap()
                .get("session")
                .is_none()
        );
    }
    for wire in [
        json!({"session": {}}),
        json!({"session": {"compaction": null}}),
    ] {
        let capabilities: ClientCapabilities = serde_json::from_value(wire).unwrap();
        assert!(capabilities.session.as_ref().unwrap().compaction.is_none());
        assert_eq!(
            serde_json::to_value(capabilities).unwrap()["session"],
            json!({})
        );
    }
    let capabilities = ClientCapabilities::new()
        .session(ClientSessionCapabilities::new().compaction(CompactionCapabilities::new()));
    let wire = serde_json::to_value(&capabilities).unwrap();
    assert_eq!(wire["session"], json!({"compaction": {}}));
    assert_eq!(
        serde_json::from_value::<ClientCapabilities>(wire).unwrap(),
        capabilities
    );
}
