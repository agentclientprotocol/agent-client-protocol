//! Custom payload adapters, option-like field wrappers, and builder helpers for serde.
//!
//! ## Payload adapters
//!
//! - [`default_on_null`] — opt a defaultable payload into accepting null.
//!
//! ## Types
//!
//! - [`MaybeUndefined<T>`] — three-state: undefined (key absent), null, or value.
//! - [`SkipListener`] — [`serde_with::InspectError`] hook used by every
//!   `VecSkipError` call site in the protocol types.
//!
//! ## Builder traits
//!
//! - [`IntoOption<T>`] — ergonomic conversion into `Option<T>` for builder methods.
//! - [`IntoMaybeUndefined<T>`] — ergonomic conversion into `MaybeUndefined<T>` for builder methods.
//!
//! `MaybeUndefined` based on: <https://docs.rs/async-graphql/latest/src/async_graphql/types/maybe_undefined.rs.html>
use std::{
    borrow::Cow,
    ffi::OsStr,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{DeserializeAs, de::DeserializeAsWrap};

// ---- Default-on-null payloads ----

/// Declares a defaultable payload whose `Deserialize` accepts null.
///
/// Declare the normal derives except `Deserialize` inside this macro. A private
/// wire type derives deserialization from the same fields and attributes, so
/// there is no second field definition to maintain and no public helper methods.
/// `DefaultOnNull` wraps only deserialization of the entire payload; serialization
/// and JSON Schema are still derived directly on the public type.
///
/// Opt in explicitly; implementing `Default` alone does not change wire behavior.
macro_rules! default_on_null {
    (
        $(#[$attribute:meta])*
        $visibility:vis struct $payload:ident {
            $(
                $(#[$field_attribute:meta])*
                $field_visibility:vis $field:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$attribute])*
        $visibility struct $payload {
            $(
                $(#[$field_attribute])*
                $field_visibility $field: $field_type,
            )*
        }

        impl<'de> serde::Deserialize<'de> for $payload {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                $(#[$attribute])*
                #[derive(serde::Deserialize)]
                struct Wire {
                    $(
                        $(#[$field_attribute])*
                        $field_visibility $field: $field_type,
                    )*
                }

                struct NonNull;

                impl<'de> serde_with::DeserializeAs<'de, $payload> for NonNull {
                    fn deserialize_as<D: serde::Deserializer<'de>>(
                        deserializer: D,
                    ) -> Result<$payload, D::Error> {
                        let wire = <Wire as serde::Deserialize>::deserialize(deserializer)?;
                        Ok($payload {
                            $($field: wire.$field,)*
                        })
                    }
                }

                <serde_with::DefaultOnNull<NonNull> as serde_with::DeserializeAs<
                    'de,
                    Self,
                >>::deserialize_as(deserializer)
            }
        }
    };
}

pub(crate) use default_on_null;

#[cfg(test)]
mod default_on_null_tests {
    use serde::{Deserialize, Serialize, de::DeserializeOwned};
    use serde_json::{Value, json};

    use crate::{
        MaybeUndefined,
        rpc::{JsonRpcMessage, Request, Response},
        v1::{
            self, LoadSessionResponse, NewSessionResponse, PromptResponse, ReadTextFileResponse,
            RequestPermissionResponse, WaitForTerminalExitResponse, WriteTextFileResponse,
        },
    };

    // Keep one inventory for the value, streaming, serialization, and schema checks.
    // Feature gates match the payloads, so the inventory also runs without default
    // features and with individual unstable features enabled.
    macro_rules! for_each_defaultable_payload {
        ($check:ident) => {
            $check::<v1::AuthenticateResponse>();
            $check::<v1::LogoutRequest>();
            $check::<v1::LogoutResponse>();
            $check::<v1::LoadSessionResponse>();
            $check::<v1::ResumeSessionResponse>();
            $check::<v1::CloseSessionResponse>();
            $check::<v1::ListSessionsRequest>();
            $check::<v1::DeleteSessionResponse>();
            $check::<v1::SetSessionModeResponse>();
            $check::<v1::WriteTextFileResponse>();
            $check::<v1::ReleaseTerminalResponse>();
            $check::<v1::KillTerminalResponse>();
            $check::<v1::WaitForTerminalExitResponse>();

            #[cfg(feature = "unstable_llm_providers")]
            {
                $check::<v1::ListProvidersRequest>();
                $check::<v1::SetProviderResponse>();
                $check::<v1::DisableProviderResponse>();
            }
            #[cfg(feature = "unstable_nes")]
            {
                $check::<v1::StartNesRequest>();
                $check::<v1::CloseNesResponse>();
            }
            #[cfg(feature = "unstable_mcp_over_acp")]
            {
                $check::<v1::DisconnectMcpResponse>();
            }

            #[cfg(feature = "unstable_protocol_v2")]
            {
                use crate::v2;

                $check::<v2::LoginAuthResponse>();
                $check::<v2::LogoutAuthRequest>();
                $check::<v2::LogoutAuthResponse>();
                $check::<v2::ResumeSessionResponse>();
                $check::<v2::CloseSessionResponse>();
                $check::<v2::ListSessionsRequest>();
                $check::<v2::DeleteSessionResponse>();

                #[cfg(feature = "unstable_llm_providers")]
                {
                    $check::<v2::ListProvidersRequest>();
                    $check::<v2::SetProviderResponse>();
                    $check::<v2::DisableProviderResponse>();
                }
                #[cfg(feature = "unstable_nes")]
                {
                    $check::<v2::StartNesRequest>();
                    $check::<v2::CloseNesResponse>();
                }
                #[cfg(feature = "unstable_mcp_over_acp")]
                {
                    $check::<v2::DisconnectMcpResponse>();
                }
            }
        };
    }

    fn assert_defaultable_payload<T>()
    where
        T: Default + DeserializeOwned + Serialize + PartialEq + std::fmt::Debug,
    {
        let name = std::any::type_name::<T>();
        for value in [Value::Null, json!({})] {
            assert_eq!(
                serde_json::from_value::<T>(value).unwrap(),
                T::default(),
                "{name}",
            );
        }
        assert_eq!(
            serde_json::from_str::<T>("null").unwrap(),
            T::default(),
            "{name}",
        );
        assert_eq!(
            serde_json::to_value(T::default()).unwrap(),
            json!({}),
            "{name}"
        );

        let metadata = json!({"_meta": {"example.com/key": ["preserve", 1, null]}});
        let payload: T = serde_json::from_value(metadata.clone()).unwrap();
        assert_eq!(serde_json::to_value(payload).unwrap(), metadata, "{name}");

        for value in [json!(false), json!(42), json!("invalid")] {
            assert!(serde_json::from_value::<T>(value).is_err(), "{name}");
        }

        // Opting in the payload must not swallow null in surrounding wrappers.
        assert_eq!(
            serde_json::from_value::<Option<T>>(Value::Null).unwrap(),
            None,
            "{name}",
        );
        assert_eq!(
            serde_json::from_value::<MaybeUndefined<T>>(Value::Null).unwrap(),
            MaybeUndefined::Null,
            "{name}",
        );
    }

    #[test]
    fn defaultable_payloads_accept_null_without_losing_information() {
        for_each_defaultable_payload!(assert_defaultable_payload);
    }

    #[test]
    fn inherent_methods_do_not_bypass_null_handling() {
        // These must resolve to the trait too, not a strict inherent helper.
        assert_eq!(
            WriteTextFileResponse::deserialize(Value::Null).unwrap(),
            WriteTextFileResponse::default(),
        );
        assert_eq!(
            LoadSessionResponse::deserialize(Value::Null).unwrap(),
            LoadSessionResponse::default(),
        );
    }

    #[test]
    fn optional_payload_fields_are_preserved() {
        let load = json!({
            "modes": {
                "currentModeId": "ask",
                "availableModes": [{"id": "ask", "name": "Ask"}]
            },
            "configOptions": [],
            "_meta": {"example.com/key": "value"}
        });
        let response: LoadSessionResponse = serde_json::from_value(load.clone()).unwrap();
        assert_eq!(serde_json::to_value(response).unwrap(), load);

        let list = json!({"cwd": "/workspace", "cursor": "next-page"});
        let request: v1::ListSessionsRequest = serde_json::from_value(list.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), list);

        #[cfg(feature = "unstable_protocol_v2")]
        {
            let request: crate::v2::ListSessionsRequest =
                serde_json::from_value(list.clone()).unwrap();
            assert_eq!(serde_json::to_value(request).unwrap(), list);
        }
    }

    #[test]
    fn default_terminal_exit_status_is_unknown_not_success() {
        let response: WaitForTerminalExitResponse = serde_json::from_value(Value::Null).unwrap();
        assert_eq!(response.exit_status.exit_code, None);
        assert_eq!(response.exit_status.signal, None);
        assert_eq!(response, WaitForTerminalExitResponse::default());

        for value in [
            json!({"exitCode": 0}),
            json!({"exitCode": 17}),
            json!({"signal": "SIGTERM"}),
        ] {
            let response: WaitForTerminalExitResponse =
                serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(response).unwrap(), value);
        }
    }

    #[test]
    fn non_null_payloads_keep_the_derived_deserialization_behavior() {
        // A default exists, but the field is still required in non-null input.
        // DefaultOnNull must not become DefaultOnError.
        super::default_on_null! {
            #[derive(Default, Debug, Serialize, PartialEq)]
            struct RequiredField {
                count: u32,
            }
        }
        #[derive(Deserialize)]
        struct BaselineWrite {
            #[serde(
                default,
                rename = "_meta",
                with = "serde_with::As::<serde_with::DefaultOnError>"
            )]
            meta: Option<serde_json::Map<String, Value>>,
        }

        assert_eq!(
            serde_json::from_value::<RequiredField>(Value::Null).unwrap(),
            RequiredField::default(),
        );
        assert!(serde_json::from_value::<RequiredField>(json!({})).is_err());
        assert!(serde_json::from_value::<RequiredField>(json!({"count": "invalid"})).is_err());

        for value in [
            json!({}),
            json!({"_meta": {"example.com/key": true}}),
            json!({"_meta": 42}),
            json!({"modes": "invalid", "configOptions": "invalid"}),
            json!([]),
            json!([null]),
            json!(false),
            json!(42),
            json!("invalid"),
        ] {
            assert_eq!(
                serde_json::from_value::<WriteTextFileResponse>(value.clone())
                    .map(|response| response.meta)
                    .map_err(|_| ()),
                serde_json::from_value::<BaselineWrite>(value)
                    .map(|response| response.meta)
                    .map_err(|_| ()),
            );
        }
    }

    #[test]
    fn payloads_with_required_fields_still_reject_null() {
        assert!(serde_json::from_value::<ReadTextFileResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<RequestPermissionResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<NewSessionResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<PromptResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<v1::InitializeResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<v1::CreateTerminalResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<v1::TerminalOutputResponse>(Value::Null).is_err());
        assert!(serde_json::from_value::<v1::CreateElicitationResponse>(Value::Null).is_err());

        #[cfg(feature = "unstable_protocol_v2")]
        {
            use crate::v2;

            assert!(serde_json::from_value::<v2::InitializeResponse>(Value::Null).is_err());
            assert!(serde_json::from_value::<v2::NewSessionResponse>(Value::Null).is_err());
            assert!(serde_json::from_value::<v2::PromptResponse>(Value::Null).is_err());
            assert!(serde_json::from_value::<v2::RequestPermissionResponse>(Value::Null).is_err());
            assert!(serde_json::from_value::<v2::CreateElicitationResponse>(Value::Null).is_err());
        }
    }

    #[test]
    fn raw_response_nulls_are_not_rewritten() {
        let extension: v1::ExtResponse = serde_json::from_value(Value::Null).unwrap();
        assert_eq!(serde_json::to_value(extension).unwrap(), Value::Null);

        #[cfg(feature = "unstable_mcp_over_acp")]
        {
            let mcp: v1::MessageMcpResponse = serde_json::from_value(Value::Null).unwrap();
            assert_eq!(serde_json::to_value(mcp).unwrap(), Value::Null);
        }

        #[cfg(feature = "unstable_protocol_v2")]
        {
            let extension: crate::v2::ExtResponse = serde_json::from_value(Value::Null).unwrap();
            assert_eq!(serde_json::to_value(extension).unwrap(), Value::Null);

            #[cfg(feature = "unstable_mcp_over_acp")]
            {
                let mcp: crate::v2::MessageMcpResponse =
                    serde_json::from_value(Value::Null).unwrap();
                assert_eq!(serde_json::to_value(mcp).unwrap(), Value::Null);
            }
        }
    }

    #[test]
    fn optional_request_parameters_keep_their_existing_meaning() {
        type LogoutRequest = Request<v1::LogoutRequest>;
        for value in [
            json!({"id": 1, "method": "logout"}),
            json!({"id": 1, "method": "logout", "params": null}),
        ] {
            let request: LogoutRequest = serde_json::from_value(value).unwrap();
            assert_eq!(request.params, None);
        }
        let request: LogoutRequest =
            serde_json::from_value(json!({"id": 1, "method": "logout", "params": {}})).unwrap();
        assert_eq!(request.params, Some(v1::LogoutRequest::default()));
    }

    #[test]
    fn nullable_fields_keep_their_existing_meaning() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Container {
            optional: Option<WriteTextFileResponse>,
            #[serde(default)]
            patch: MaybeUndefined<WriteTextFileResponse>,
        }

        assert_eq!(
            serde_json::from_value::<Option<WriteTextFileResponse>>(Value::Null).unwrap(),
            None,
        );
        assert_eq!(
            serde_json::from_value::<MaybeUndefined<WriteTextFileResponse>>(Value::Null).unwrap(),
            MaybeUndefined::Null,
        );
        assert_eq!(
            serde_json::from_value::<Container>(json!({})).unwrap(),
            Container {
                optional: None,
                patch: MaybeUndefined::Undefined,
            },
        );
        assert_eq!(
            serde_json::from_value::<Container>(json!({"optional": null, "patch": null})).unwrap(),
            Container {
                optional: None,
                patch: MaybeUndefined::Null,
            },
        );
        assert_eq!(
            serde_json::from_value::<Container>(json!({"optional": {}, "patch": {}})).unwrap(),
            Container {
                optional: Some(WriteTextFileResponse::default()),
                patch: MaybeUndefined::Value(WriteTextFileResponse::default()),
            },
        );
    }

    #[test]
    fn response_result_is_required_even_when_its_payload_accepts_null() {
        type WriteResponse = Response<WriteTextFileResponse, Value>;
        let response: WriteResponse =
            serde_json::from_value(json!({"id": 1, "result": null})).unwrap();
        assert_eq!(
            response,
            Response::new(1, Ok(WriteTextFileResponse::default())),
        );
        assert!(serde_json::from_value::<WriteResponse>(json!({"id": 1})).is_err());
        assert!(
            serde_json::from_value::<JsonRpcMessage<WriteResponse>>(
                json!({"jsonrpc": "2.0", "id": 1})
            )
            .is_err()
        );

        let error = json!({"code": -32603, "message": "Internal error"});
        let response: JsonRpcMessage<WriteResponse> = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": error.clone()
        }))
        .unwrap();
        assert_eq!(response.into_inner(), Response::new(1, Err(error)));
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn defaultable_payload_schemas_still_require_objects() {
        fn assert_object_schema<T: schemars::JsonSchema>() {
            let schema = serde_json::to_value(schemars::schema_for!(T)).unwrap();
            assert_eq!(schema["type"], "object");
            assert!(schema.get("anyOf").is_none());
        }
        for_each_defaultable_payload!(assert_object_schema);
    }
}

// ---- SkipListener ----

/// Inspector passed to every `VecSkipError<_, SkipListener>` in the protocol
/// types so that malformed list entries dropped during deserialization are
/// surfaced to observability tooling rather than vanishing silently.
///
/// - With the `tracing` feature enabled, this is a zero-sized type whose
///   [`InspectError`](serde_with::InspectError) implementation emits a
///   [`tracing::warn!`] event on every skipped entry.
/// - With the feature disabled (the default), it resolves to `()` — which
///   `serde_with` ships with a no-op `InspectError` implementation — so call
///   sites incur zero runtime cost.
#[cfg(feature = "tracing")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub(crate) struct SkipListener;

#[cfg(feature = "tracing")]
impl serde_with::InspectError for SkipListener {
    fn inspect_error(error: impl serde::de::Error) {
        tracing::warn!(
            %error,
            "skipped malformed list entry during deserialization",
        );
    }
}

/// Zero-cost stand-in for [`SkipListener`] when the `tracing` feature is
/// disabled. Resolves to `()`, which `serde_with` already ships with a no-op
/// `InspectError` implementation.
#[cfg(not(feature = "tracing"))]
pub(crate) type SkipListener = ();

#[cfg(test)]
mod skip_listener_tests {
    use std::cell::Cell;

    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use serde_with::{DefaultOnError, VecSkipError, serde_as};

    thread_local! {
        static SKIP_COUNT: Cell<u32> = const { Cell::new(0) };
    }

    /// Test-only inspector that counts skipped entries.
    struct CountingListener;

    impl serde_with::InspectError for CountingListener {
        fn inspect_error(_error: impl serde::de::Error) {
            SKIP_COUNT.with(|c| c.set(c.get() + 1));
        }
    }

    #[serde_as]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Wrapper {
        #[serde_as(deserialize_as = "VecSkipError<_, CountingListener>")]
        values: Vec<u32>,
    }

    #[test]
    fn inspector_runs_for_each_skipped_entry() {
        SKIP_COUNT.with(|c| c.set(0));

        let input = json!({"values": [1, "oops", 2, {}, 3]});
        let wrapper: Wrapper = serde_json::from_value(input).unwrap();

        assert_eq!(wrapper.values, vec![1, 2, 3]);
        assert_eq!(SKIP_COUNT.with(Cell::get), 2);
    }

    /// Mirrors the pattern applied to every required `Vec<T>` field in the
    /// protocol: `DefaultOnError<VecSkipError<_, ...>>` + `#[serde(default)]`.
    /// Element-level failures are skipped; any outer shape error (`null`, a
    /// string, a map, etc.) collapses to `Default::default()` (i.e. `vec![]`).
    #[serde_as]
    #[derive(Deserialize, Debug, PartialEq)]
    struct ResilientVec {
        #[serde_as(deserialize_as = "DefaultOnError<VecSkipError<_, CountingListener>>")]
        #[serde(default)]
        values: Vec<u32>,
    }

    #[test]
    fn resilient_vec_tolerates_missing_null_and_wrong_type() {
        // Missing field -> `#[serde(default)]` supplies `vec![]`.
        let r: ResilientVec = serde_json::from_value(json!({})).unwrap();
        assert_eq!(r.values, Vec::<u32>::new());

        // Explicit null -> `DefaultOnError` swallows the type error.
        let r: ResilientVec = serde_json::from_value(json!({"values": null})).unwrap();
        assert_eq!(r.values, Vec::<u32>::new());

        // Wrong outer type (string) -> `DefaultOnError` swallows.
        let r: ResilientVec = serde_json::from_value(json!({"values": "oops"})).unwrap();
        assert_eq!(r.values, Vec::<u32>::new());

        // Wrong outer type (object) -> `DefaultOnError` swallows.
        let r: ResilientVec = serde_json::from_value(json!({"values": {"k": 1}})).unwrap();
        assert_eq!(r.values, Vec::<u32>::new());

        // Valid array with element errors -> `VecSkipError` skips per-element.
        SKIP_COUNT.with(|c| c.set(0));
        let r: ResilientVec =
            serde_json::from_value(json!({"values": [1, "oops", 2, {}, 3]})).unwrap();
        assert_eq!(r.values, vec![1, 2, 3]);
        assert_eq!(SKIP_COUNT.with(Cell::get), 2);
    }

    #[test]
    fn resilient_vec_does_not_invoke_inspector_on_outer_failure() {
        SKIP_COUNT.with(|c| c.set(0));

        // Outer failures are swallowed silently by `DefaultOnError`; the
        // inspector only sees per-element failures inside a valid array.
        let _r: ResilientVec = serde_json::from_value(json!({"values": null})).unwrap();
        let _r: ResilientVec = serde_json::from_value(json!({"values": "oops"})).unwrap();
        let _r: ResilientVec = serde_json::from_value(json!({"values": {}})).unwrap();

        assert_eq!(SKIP_COUNT.with(Cell::get), 0);
    }

    /// Mirrors the pattern applied to every optional `Option<Vec<T>>` field:
    /// `DefaultOnError<Option<VecSkipError<_, ...>>>` + `#[serde(default)]`.
    /// `null` becomes `None`; outer shape errors also collapse to `None`;
    /// element-level failures are skipped inside the array.
    #[serde_as]
    #[derive(Deserialize, Debug, PartialEq)]
    struct ResilientOptionVec {
        #[serde_as(deserialize_as = "DefaultOnError<Option<VecSkipError<_, CountingListener>>>")]
        #[serde(default)]
        values: Option<Vec<u32>>,
    }

    #[test]
    fn resilient_option_vec_tolerates_missing_null_and_wrong_type() {
        // Missing field -> `None`.
        let r: ResilientOptionVec = serde_json::from_value(json!({})).unwrap();
        assert_eq!(r.values, None);

        // Explicit null -> `None`.
        let r: ResilientOptionVec = serde_json::from_value(json!({"values": null})).unwrap();
        assert_eq!(r.values, None);

        // Empty array -> `Some(vec![])`.
        let r: ResilientOptionVec = serde_json::from_value(json!({"values": []})).unwrap();
        assert_eq!(r.values, Some(Vec::<u32>::new()));

        // Valid array -> `Some(vec)`.
        let r: ResilientOptionVec = serde_json::from_value(json!({"values": [1, 2, 3]})).unwrap();
        assert_eq!(r.values, Some(vec![1, 2, 3]));

        // Wrong outer type (string) -> `DefaultOnError` collapses to `None`.
        let r: ResilientOptionVec = serde_json::from_value(json!({"values": "oops"})).unwrap();
        assert_eq!(r.values, None);

        // Wrong outer type (object) -> `DefaultOnError` collapses to `None`.
        let r: ResilientOptionVec = serde_json::from_value(json!({"values": {"k": 1}})).unwrap();
        assert_eq!(r.values, None);

        // Valid array with element errors -> `VecSkipError` skips per-element.
        SKIP_COUNT.with(|c| c.set(0));
        let r: ResilientOptionVec =
            serde_json::from_value(json!({"values": [1, "oops", 2, {}, 3]})).unwrap();
        assert_eq!(r.values, Some(vec![1, 2, 3]));
        assert_eq!(SKIP_COUNT.with(Cell::get), 2);
    }
}

// ---- IntoOption ----

/// Utility trait for builder methods for optional values.
/// This allows the caller to either pass in the value itself without wrapping it in `Some`,
/// or to just pass in an Option if that is what they have.
pub trait IntoOption<T> {
    /// Converts this value into an optional builder argument.
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T> IntoOption<T> for T {
    fn into_option(self) -> Option<T> {
        Some(self)
    }
}

impl IntoOption<String> for &str {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for &mut str {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for &String {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for Box<str> {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for Cow<'_, str> {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}

impl IntoOption<String> for Arc<str> {
    fn into_option(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl<T: ?Sized + AsRef<OsStr>> IntoOption<PathBuf> for &T {
    fn into_option(self) -> Option<PathBuf> {
        Some(self.into())
    }
}

impl IntoOption<PathBuf> for Box<Path> {
    fn into_option(self) -> Option<PathBuf> {
        Some(self.into())
    }
}

impl IntoOption<PathBuf> for Cow<'_, Path> {
    fn into_option(self) -> Option<PathBuf> {
        Some(self.into())
    }
}

impl IntoOption<serde_json::Value> for &str {
    fn into_option(self) -> Option<serde_json::Value> {
        Some(self.into())
    }
}

impl IntoOption<serde_json::Value> for String {
    fn into_option(self) -> Option<serde_json::Value> {
        Some(self.into())
    }
}

impl IntoOption<serde_json::Value> for Cow<'_, str> {
    fn into_option(self) -> Option<serde_json::Value> {
        Some(self.into())
    }
}

// ---- MaybeUndefined ----

/// Similar to `Option`, but it has three states, `undefined`, `null` and `x`.
///
/// When using with Serde, you will likely want to skip serialization of `undefined`
/// and add a `default` for deserialization.
///
/// # Example
///
/// ```rust
/// use agent_client_protocol_schema::MaybeUndefined;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
/// struct A {
///     #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
///     a: MaybeUndefined<i32>,
/// }
/// ```
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[cfg_attr(feature = "schemars", schemars(with = "Option<Option<T>>", inline))]
#[expect(clippy::exhaustive_enums)]
pub enum MaybeUndefined<T> {
    /// The field was not present.
    #[default]
    Undefined,
    /// The field was present with a JSON `null` value.
    Null,
    /// The field was present with a non-null value.
    Value(T),
}

impl<T> MaybeUndefined<T> {
    /// Returns true if the `MaybeUndefined<T>` is undefined.
    #[inline]
    pub const fn is_undefined(&self) -> bool {
        matches!(self, MaybeUndefined::Undefined)
    }

    /// Returns true if the `MaybeUndefined<T>` is null.
    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, MaybeUndefined::Null)
    }

    /// Returns true if the `MaybeUndefined<T>` contains value.
    #[inline]
    pub const fn is_value(&self) -> bool {
        matches!(self, MaybeUndefined::Value(_))
    }

    /// Borrow the value, returns `None` if the `MaybeUndefined<T>` is
    /// `undefined` or `null`, otherwise returns `Some(T)`.
    #[inline]
    pub const fn value(&self) -> Option<&T> {
        match self {
            MaybeUndefined::Value(value) => Some(value),
            _ => None,
        }
    }

    /// Converts the `MaybeUndefined<T>` to `Option<T>`.
    #[inline]
    pub fn take(self) -> Option<T> {
        match self {
            MaybeUndefined::Value(value) => Some(value),
            _ => None,
        }
    }

    /// Converts the `MaybeUndefined<T>` to `Option<Option<T>>`.
    #[inline]
    pub const fn as_opt_ref(&self) -> Option<Option<&T>> {
        match self {
            MaybeUndefined::Undefined => None,
            MaybeUndefined::Null => Some(None),
            MaybeUndefined::Value(value) => Some(Some(value)),
        }
    }

    /// Converts the `MaybeUndefined<T>` to `Option<Option<&U>>`.
    #[inline]
    pub fn as_opt_deref<U>(&self) -> Option<Option<&U>>
    where
        U: ?Sized,
        T: Deref<Target = U>,
    {
        match self {
            MaybeUndefined::Undefined => None,
            MaybeUndefined::Null => Some(None),
            MaybeUndefined::Value(value) => Some(Some(&**value)),
        }
    }

    /// Returns `true` if the `MaybeUndefined<T>` contains the given value.
    #[inline]
    pub fn contains_value<U>(&self, x: &U) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            MaybeUndefined::Value(y) => x == y,
            _ => false,
        }
    }

    /// Returns `true` if the `MaybeUndefined<T>` contains the given nullable
    /// value.
    #[inline]
    pub fn contains<U>(&self, x: Option<&U>) -> bool
    where
        U: PartialEq<T>,
    {
        match self {
            MaybeUndefined::Value(y) => matches!(x, Some(v) if v == y),
            MaybeUndefined::Null => x.is_none(),
            MaybeUndefined::Undefined => false,
        }
    }

    /// Maps a `MaybeUndefined<T>` to `MaybeUndefined<U>` by applying a function
    /// to the contained nullable value
    #[inline]
    pub fn map<U, F: FnOnce(Option<T>) -> Option<U>>(self, f: F) -> MaybeUndefined<U> {
        match self {
            MaybeUndefined::Value(v) => match f(Some(v)) {
                Some(v) => MaybeUndefined::Value(v),
                None => MaybeUndefined::Null,
            },
            MaybeUndefined::Null => match f(None) {
                Some(v) => MaybeUndefined::Value(v),
                None => MaybeUndefined::Null,
            },
            MaybeUndefined::Undefined => MaybeUndefined::Undefined,
        }
    }

    /// Maps a `MaybeUndefined<T>` to `MaybeUndefined<U>` by applying a function
    /// to the contained value
    #[inline]
    pub fn map_value<U, F: FnOnce(T) -> U>(self, f: F) -> MaybeUndefined<U> {
        match self {
            MaybeUndefined::Value(v) => MaybeUndefined::Value(f(v)),
            MaybeUndefined::Null => MaybeUndefined::Null,
            MaybeUndefined::Undefined => MaybeUndefined::Undefined,
        }
    }

    /// Update `value` if the `MaybeUndefined<T>` is not undefined.
    ///
    /// # Example
    ///
    /// ```rust
    /// use agent_client_protocol_schema::MaybeUndefined;
    ///
    /// let mut value = None;
    ///
    /// MaybeUndefined::Value(10i32).update_to(&mut value);
    /// assert_eq!(value, Some(10));
    ///
    /// MaybeUndefined::Undefined.update_to(&mut value);
    /// assert_eq!(value, Some(10));
    ///
    /// MaybeUndefined::Null.update_to(&mut value);
    /// assert_eq!(value, None);
    /// ```
    pub fn update_to(self, value: &mut Option<T>) {
        match self {
            MaybeUndefined::Value(new) => *value = Some(new),
            MaybeUndefined::Null => *value = None,
            MaybeUndefined::Undefined => {}
        }
    }
}

impl<T, E> MaybeUndefined<Result<T, E>> {
    /// Transposes a `MaybeUndefined` of a [`Result`] into a [`Result`] of a
    /// `MaybeUndefined`.
    ///
    /// [`MaybeUndefined::Undefined`] will be mapped to
    /// [`Ok`]`(`[`MaybeUndefined::Undefined`]`)`. [`MaybeUndefined::Null`]
    /// will be mapped to [`Ok`]`(`[`MaybeUndefined::Null`]`)`.
    /// [`MaybeUndefined::Value`]`(`[`Ok`]`(_))` and
    /// [`MaybeUndefined::Value`]`(`[`Err`]`(_))` will be mapped to
    /// [`Ok`]`(`[`MaybeUndefined::Value`]`(_))` and [`Err`]`(_)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is [`MaybeUndefined::Value`]`(`[`Err`]`(_))`.
    #[inline]
    pub fn transpose(self) -> Result<MaybeUndefined<T>, E> {
        match self {
            MaybeUndefined::Undefined => Ok(MaybeUndefined::Undefined),
            MaybeUndefined::Null => Ok(MaybeUndefined::Null),
            MaybeUndefined::Value(Ok(v)) => Ok(MaybeUndefined::Value(v)),
            MaybeUndefined::Value(Err(e)) => Err(e),
        }
    }
}

impl<T: Serialize> Serialize for MaybeUndefined<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MaybeUndefined::Value(value) => value.serialize(serializer),
            MaybeUndefined::Null => serializer.serialize_none(),
            MaybeUndefined::Undefined => serializer.serialize_unit(),
        }
    }
}

impl<'de, T> Deserialize<'de> for MaybeUndefined<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<MaybeUndefined<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => MaybeUndefined::Value(value),
            None => MaybeUndefined::Null,
        })
    }
}

impl<T> From<MaybeUndefined<T>> for Option<Option<T>> {
    fn from(maybe_undefined: MaybeUndefined<T>) -> Self {
        match maybe_undefined {
            MaybeUndefined::Undefined => None,
            MaybeUndefined::Null => Some(None),
            MaybeUndefined::Value(value) => Some(Some(value)),
        }
    }
}

impl<T> From<Option<Option<T>>> for MaybeUndefined<T> {
    fn from(value: Option<Option<T>>) -> Self {
        match value {
            Some(Some(value)) => Self::Value(value),
            Some(None) => Self::Null,
            None => Self::Undefined,
        }
    }
}

impl<'de, T, TAs> DeserializeAs<'de, MaybeUndefined<T>> for MaybeUndefined<TAs>
where
    TAs: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<MaybeUndefined<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<DeserializeAsWrap<T, TAs>>::deserialize(deserializer).map(|value| match value {
            Some(value) => MaybeUndefined::Value(value.into_inner()),
            None => MaybeUndefined::Null,
        })
    }
}

/// Utility trait for builder methods for optional values.
/// This allows the caller to either pass in the value itself without wrapping it in `Some`,
/// or to just pass in an Option if that is what they have, or set it back to undefined.
pub trait IntoMaybeUndefined<T> {
    /// Converts this value into a three-state builder argument.
    fn into_maybe_undefined(self) -> MaybeUndefined<T>;
}

impl<T> IntoMaybeUndefined<T> for T {
    fn into_maybe_undefined(self) -> MaybeUndefined<T> {
        MaybeUndefined::Value(self)
    }
}

impl<T> IntoMaybeUndefined<T> for Option<T> {
    fn into_maybe_undefined(self) -> MaybeUndefined<T> {
        match self {
            Some(value) => MaybeUndefined::Value(value),
            None => MaybeUndefined::Null,
        }
    }
}

impl<T> IntoMaybeUndefined<T> for MaybeUndefined<T> {
    fn into_maybe_undefined(self) -> MaybeUndefined<T> {
        self
    }
}

impl IntoMaybeUndefined<String> for &str {
    fn into_maybe_undefined(self) -> MaybeUndefined<String> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<String> for &mut str {
    fn into_maybe_undefined(self) -> MaybeUndefined<String> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<String> for &String {
    fn into_maybe_undefined(self) -> MaybeUndefined<String> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<String> for Box<str> {
    fn into_maybe_undefined(self) -> MaybeUndefined<String> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<String> for Cow<'_, str> {
    fn into_maybe_undefined(self) -> MaybeUndefined<String> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<String> for Arc<str> {
    fn into_maybe_undefined(self) -> MaybeUndefined<String> {
        MaybeUndefined::Value(self.to_string())
    }
}

impl<T: ?Sized + AsRef<OsStr>> IntoMaybeUndefined<PathBuf> for &T {
    fn into_maybe_undefined(self) -> MaybeUndefined<PathBuf> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<PathBuf> for Box<Path> {
    fn into_maybe_undefined(self) -> MaybeUndefined<PathBuf> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<PathBuf> for Cow<'_, Path> {
    fn into_maybe_undefined(self) -> MaybeUndefined<PathBuf> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<serde_json::Value> for &str {
    fn into_maybe_undefined(self) -> MaybeUndefined<serde_json::Value> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<serde_json::Value> for String {
    fn into_maybe_undefined(self) -> MaybeUndefined<serde_json::Value> {
        MaybeUndefined::Value(self.into())
    }
}

impl IntoMaybeUndefined<serde_json::Value> for Cow<'_, str> {
    fn into_maybe_undefined(self) -> MaybeUndefined<serde_json::Value> {
        MaybeUndefined::Value(self.into())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::{from_value, json, to_value};

    use super::*;

    #[test]
    fn test_maybe_undefined_serde() {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct A {
            #[serde(default, skip_serializing_if = "MaybeUndefined::is_undefined")]
            a: MaybeUndefined<i32>,
        }

        assert_eq!(to_value(MaybeUndefined::Value(100i32)).unwrap(), json!(100));

        assert_eq!(
            from_value::<MaybeUndefined<i32>>(json!(100)).unwrap(),
            MaybeUndefined::Value(100)
        );
        assert_eq!(
            from_value::<MaybeUndefined<i32>>(json!(null)).unwrap(),
            MaybeUndefined::Null
        );

        assert_eq!(
            to_value(&A {
                a: MaybeUndefined::Value(100i32)
            })
            .unwrap(),
            json!({"a": 100})
        );

        assert_eq!(
            to_value(&A {
                a: MaybeUndefined::Null,
            })
            .unwrap(),
            json!({ "a": null })
        );

        assert_eq!(
            to_value(&A {
                a: MaybeUndefined::Undefined,
            })
            .unwrap(),
            json!({})
        );

        assert_eq!(
            from_value::<A>(json!({"a": 100})).unwrap(),
            A {
                a: MaybeUndefined::Value(100i32)
            }
        );

        assert_eq!(
            from_value::<A>(json!({ "a": null })).unwrap(),
            A {
                a: MaybeUndefined::Null
            }
        );

        assert_eq!(
            from_value::<A>(json!({})).unwrap(),
            A {
                a: MaybeUndefined::Undefined
            }
        );
    }

    #[test]
    fn test_maybe_undefined_to_nested_option() {
        assert_eq!(Option::<Option<i32>>::from(MaybeUndefined::Undefined), None);

        assert_eq!(
            Option::<Option<i32>>::from(MaybeUndefined::Null),
            Some(None)
        );

        assert_eq!(
            Option::<Option<i32>>::from(MaybeUndefined::Value(42)),
            Some(Some(42))
        );
    }

    #[test]
    fn test_as_opt_ref() {
        let value = MaybeUndefined::<String>::Undefined;
        let r = value.as_opt_ref();
        assert_eq!(r, None);

        let value = MaybeUndefined::<String>::Null;
        let r = value.as_opt_ref();
        assert_eq!(r, Some(None));

        let value = MaybeUndefined::<String>::Value("abc".to_string());
        let r = value.as_opt_ref();
        assert_eq!(r, Some(Some(&"abc".to_string())));
    }

    #[test]
    fn test_as_opt_deref() {
        let value = MaybeUndefined::<String>::Undefined;
        let r = value.as_opt_deref();
        assert_eq!(r, None);

        let value = MaybeUndefined::<String>::Null;
        let r = value.as_opt_deref();
        assert_eq!(r, Some(None));

        let value = MaybeUndefined::<String>::Value("abc".to_string());
        let r = value.as_opt_deref();
        assert_eq!(r, Some(Some("abc")));
    }

    #[test]
    fn test_contains_value() {
        let test = "abc";

        let mut value: MaybeUndefined<String> = MaybeUndefined::Undefined;
        assert!(!value.contains_value(&test));

        value = MaybeUndefined::Null;
        assert!(!value.contains_value(&test));

        value = MaybeUndefined::Value("abc".to_string());
        assert!(value.contains_value(&test));
    }

    #[test]
    fn test_contains() {
        let test = Some("abc");
        let none: Option<&str> = None;

        let mut value: MaybeUndefined<String> = MaybeUndefined::Undefined;
        assert!(!value.contains(test.as_ref()));
        assert!(!value.contains(none.as_ref()));

        value = MaybeUndefined::Null;
        assert!(!value.contains(test.as_ref()));
        assert!(value.contains(none.as_ref()));

        value = MaybeUndefined::Value("abc".to_string());
        assert!(value.contains(test.as_ref()));
        assert!(!value.contains(none.as_ref()));
    }

    #[test]
    fn test_map_value() {
        let mut value: MaybeUndefined<i32> = MaybeUndefined::Undefined;
        assert_eq!(value.map_value(|v| v > 2), MaybeUndefined::Undefined);

        value = MaybeUndefined::Null;
        assert_eq!(value.map_value(|v| v > 2), MaybeUndefined::Null);

        value = MaybeUndefined::Value(5);
        assert_eq!(value.map_value(|v| v > 2), MaybeUndefined::Value(true));
    }

    #[test]
    fn test_map() {
        let mut value: MaybeUndefined<i32> = MaybeUndefined::Undefined;
        assert_eq!(value.map(|v| Some(v.is_some())), MaybeUndefined::Undefined);

        value = MaybeUndefined::Null;
        assert_eq!(
            value.map(|v| Some(v.is_some())),
            MaybeUndefined::Value(false)
        );

        value = MaybeUndefined::Value(5);
        assert_eq!(
            value.map(|v| Some(v.is_some())),
            MaybeUndefined::Value(true)
        );
    }

    #[test]
    fn test_transpose() {
        let mut value: MaybeUndefined<Result<i32, &'static str>> = MaybeUndefined::Undefined;
        assert_eq!(value.transpose(), Ok(MaybeUndefined::Undefined));

        value = MaybeUndefined::Null;
        assert_eq!(value.transpose(), Ok(MaybeUndefined::Null));

        value = MaybeUndefined::Value(Ok(5));
        assert_eq!(value.transpose(), Ok(MaybeUndefined::Value(5)));

        value = MaybeUndefined::Value(Err("error"));
        assert_eq!(value.transpose(), Err("error"));
    }
}
