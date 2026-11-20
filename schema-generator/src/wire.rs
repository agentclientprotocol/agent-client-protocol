//! Validation-only layer for the published v2 schemas. The original `anyOf` and
//! payload unions remain in place because SDK generators consume their structure.

use std::collections::BTreeMap;

use agent_client_protocol_schema::v2::ErrorCode;
use serde_json::{Value, json};

// Original method recipient -> wire method -> request/notification/result payload reference.
// A result belongs to its request's recipient, not the recipient of the response.
type Methods = BTreeMap<String, BTreeMap<String, BTreeMap<&'static str, String>>>;

/// Visit the payload refs of a Schemars union, not refs *inside* the payloads.
fn visit_payloads(schema: &Value, visit: &mut impl FnMut(&str)) {
    if let Some(reference) = schema["$ref"].as_str() {
        visit(reference);
    } else {
        for key in ["allOf", "anyOf", "oneOf"] {
            if let Some(variants) = schema[key].as_array() {
                for variant in variants {
                    visit_payloads(variant, visit);
                }
            }
        }
    }
}

fn methods(schema: &Value) -> Methods {
    let defs = &schema["$defs"];
    let mut routes = Methods::new();
    let mut collect = |union: &Value, side: &str, kind: &'static str| {
        let mut count = 0;
        visit_payloads(union, &mut |reference| {
            let name = reference
                .strip_prefix("#/$defs/")
                .expect("local payload ref");
            if matches!(name, "ExtRequest" | "ExtResponse" | "ExtNotification") {
                return; // Extensions have dynamic names and no standard payload type.
            }
            let def = &defs[name];
            assert_eq!(def["x-side"], side, "wrong handler side for {name}");
            let method = def["x-method"].as_str().expect("payload lacks x-method");
            let route = routes
                .entry(side.into())
                .or_default()
                .entry(method.into())
                .or_default();
            assert!(
                route.insert(kind, reference.into()).is_none(),
                "ambiguous {side} {kind} {method}"
            );
            count += 1;
        });
        assert!(count > 0, "missing {side} {kind} payloads");
    };
    for (sender, calls_to, replies_from) in [
        ("Agent", "client", "agent"),
        ("Client", "agent", "client"),
        #[cfg(feature = "unstable")]
        ("Proxy", "proxy", "proxy"),
    ] {
        for (suffix, kind) in [("Request", "request"), ("Notification", "notification")] {
            collect(
                &defs[format!("{sender}{suffix}")]["properties"]["params"],
                calls_to,
                kind,
            );
        }
        let result = defs[format!("{sender}Response")]["anyOf"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|v| v.pointer("/properties/result"))
            .expect("response lacks result");
        collect(result, replies_from, "result");
    }
    let protocol = schema["anyOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["title"] == "ProtocolLevel")
        .expect("missing protocol notifications");
    collect(
        &protocol["properties"]["params"],
        "protocol",
        "notification",
    );
    for methods in routes.values() {
        for (name, route) in methods {
            assert_eq!(
                route.contains_key("request"),
                route.contains_key("result"),
                "unpaired {name}"
            );
        }
    }
    routes
}

fn method_branch(schema: &Value, method: &str, reference: &str) -> Value {
    let payload = schema
        .pointer(reference.strip_prefix('#').unwrap())
        .unwrap();
    let mut branch =
        json!({"properties": {"method": {"const": method}, "params": {"$ref": reference}}});
    // Only simple all-optional parameter objects in the current inventory accept {}.
    // The validator tests check omission against the actual payload schema for *every*
    // method, including transparent aliases like proxy/initialize.
    let optional = payload["type"] == "object"
        && payload.as_object().unwrap().keys().all(|k| {
            k.starts_with("x-")
                || matches!(k.as_str(), "description" | "title" | "type" | "properties")
        });
    if !optional {
        branch["required"] = json!(["params"]);
    }
    branch
}

fn direction(schema: &Value, methods: &Methods, side: &str, anchor: &str) -> Value {
    let branches = |kind: &str| {
        let mut choices = Vec::new();
        for (method, route) in methods[side].iter().chain(methods["protocol"].iter()) {
            if let Some(reference) = route.get(kind) {
                choices.push(method_branch(schema, method, reference));
            }
        }
        if side != "proxy" {
            choices.push(json!({"properties": {"method": {"type": "string", "pattern": "^_"}}}));
        }
        json!({"oneOf": choices})
    };
    json!({
        "$anchor": anchor,
        "type": "object",
        "allOf": [{"$ref": "#rpcEnvelope"}],
        "if": {"required": ["method"], "properties": {"method": true}},
        "then": {
            "if": {"required": ["id"], "properties": {"id": true}},
            "then": branches("request"),
            "else": branches("notification")
        }
    })
}

fn envelope() -> Value {
    let params = json!({"anyOf": [{"type": "object"}, {"type": "array"}]});
    json!({
        "$anchor": "rpcEnvelope",
        "type": "object",
        "required": ["jsonrpc"],
        "properties": {"jsonrpc": {"const": "2.0"}},
        "oneOf": [
            {"required": ["method", "id"], "properties": {
                "method": {"type": "string"}, "id": {"$ref": "#/$defs/RequestId"},
                "params": params, "result": false, "error": false
            }},
            {"required": ["method"], "properties": {
                "method": {"type": "string"}, "id": false, "params": params, "result": false, "error": false
            }},
            {"required": ["id", "result"], "properties": {
                "id": {"$ref": "#/$defs/RequestId"}, "result": true, "method": false, "error": false
            }},
            {"required": ["id", "error"], "properties": {
                "id": {"anyOf": [{"$ref": "#/$defs/RequestId"}, {"type": "null"}]},
                "error": {"$ref": "#/$defs/Error"}, "method": false, "result": false
            }}
        ]
    })
}

pub(super) fn add_contract(schema: &mut Value) {
    let methods = methods(schema);
    let directions = [
        direction(schema, &methods, "agent", "clientToAgent"),
        direction(schema, &methods, "client", "agentToClient"),
        #[cfg(feature = "unstable")]
        direction(schema, &methods, "proxy", "proxyExchange"),
    ];
    // Expose the same routing data for result validation by originating request.
    schema["x-methods"] = serde_json::to_value(methods).unwrap();
    schema["allOf"] = json!([{
        "$defs": {"Envelope": envelope()},
        "anyOf": directions
    }]);
    // -32002 is also used for resources other than sessions, so its data can't
    // be constrained by code alone. -32022 always specifies this data shape.
    let error = &mut schema["$defs"]["Error"];
    error["if"] = json!({"properties": {"code": {"const": i32::from(ErrorCode::UnsupportedProtocolVersion)}}});
    error["then"] = json!({"required": ["data"], "properties": {"data": {"$ref": "#/$defs/UnsupportedProtocolVersionErrorData"}}});
}
