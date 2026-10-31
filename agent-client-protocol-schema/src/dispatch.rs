//! Method-aware deserialization for the protocol request, notification, and
//! response enums.
//!
//! The wire enums ([`v1::ClientRequest`], [`v1::AgentRequest`],
//! [`v1::ClientResponse`], [`v1::AgentResponse`], [`v1::ClientNotification`],
//! [`v1::AgentNotification`], [`v1::ProtocolLevelNotification`], and their
//! [`v2`] counterparts) are `#[serde(untagged)]` so that the JSON schema
//! generator can describe each variant's payload shape. That representation
//! is **not** suitable for parsing incoming messages: serde tries the
//! variants in declaration order, so any variant whose fields all happen to
//! be optional matches *every* payload, silently dropping the data that
//! belonged to the intended variant. For example, a `session/new` request
//! used to deserialize as [`v1::ClientRequest::LogoutRequest`] with an empty
//! payload instead of failing or routing correctly.
//!
//! The `from_method_and_params` / `from_method_and_result` helpers added here
//! dispatch on the JSON-RPC method name first and then deserialize the
//! payload into the matching variant, mirroring how conformant
//! implementations route messages. Extension methods (names starting with
//! `_`) are routed to the `Ext*` variants with their raw payload preserved.

/// Error returned by the `from_method_and_params` and `from_method_and_result`
/// deserialization helpers.
#[derive(Debug)]
#[non_exhaustive]
pub enum MethodDispatchError {
    /// The method name is not part of the protocol and, where the enum has an
    /// extension variant, does not start with the extension prefix `_`.
    UnknownMethod {
        /// The unrecognized method name.
        method: String,
    },
    /// The payload did not match the shape required by the method's variant.
    InvalidPayload {
        /// The method that was dispatched.
        method: String,
        /// The underlying deserialization error.
        source: serde_json::Error,
    },
}

impl std::fmt::Display for MethodDispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMethod { method } => {
                write!(f, "unknown or unsupported method `{method}`")
            }
            Self::InvalidPayload { method, source } => {
                write!(f, "invalid payload for method `{method}`: {source}")
            }
        }
    }
}

impl std::error::Error for MethodDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownMethod { .. } => None,
            Self::InvalidPayload { source, .. } => Some(source),
        }
    }
}

macro_rules! impl_acp_from_method_and_params {
    (
        enum = $Enum:ty,
        $(ext = { $ExtVariant:ident, $ext_wrap:ident, $ExtCtor:path },)?
        arms = { $( $(#[$meta:meta])* $Variant:ident ($wrap:ident) => $method:expr ),* $(,)? }
    ) => {
        impl $Enum {
            /// Deserializes a wire payload by dispatching on the JSON-RPC
            /// method name.
            ///
            /// Unlike the derived `Deserialize` implementation (which is
            /// `#[serde(untagged)]` and exists for schema generation), this
            /// never silently selects the wrong variant: unknown non-extension
            /// methods fail with [`MethodDispatchError::UnknownMethod`], and
            /// payloads that do not match the method's shape fail with
            /// [`MethodDispatchError::InvalidPayload`].
            ///
            /// # Errors
            ///
            /// Returns [`MethodDispatchError::UnknownMethod`] for method names
            /// that are not part of the protocol and do not start with the
            /// extension prefix `_`, and [`MethodDispatchError::InvalidPayload`]
            /// when the payload does not match the dispatched variant's shape.
            ///
            /// `params` is the optional `params` member of the JSON-RPC
            /// request or notification; `None` and `null` are treated as an
            /// empty object. Extension methods (names starting with `_`) are
            /// routed to the enum's extension variant with the raw payload
            /// preserved.
            pub fn from_method_and_params(
                method: &str,
                params: Option<serde_json::Value>,
            ) -> ::std::result::Result<Self, $crate::dispatch::MethodDispatchError> {
                match method {
                    $(
                        $(#[$meta])*
                        m if m == $method => ::serde_json::from_value(
                            params.filter(|p| !p.is_null()).unwrap_or_else(|| ::serde_json::json!({})),
                        )
                        .map(|payload| Self::$Variant(impl_acp_from_method_and_params!(@wrap $wrap payload)))
                        .map_err(|source| $crate::dispatch::MethodDispatchError::InvalidPayload {
                            method: method.to_owned(),
                            source,
                        }),
                    )*
                    _ => {
                        $(
                            if method.starts_with('_') {
                                let raw_params = params.unwrap_or(::serde_json::Value::Null);
                                return ::serde_json::value::RawValue::from_string(raw_params.to_string())
                                    .map_err(|source| $crate::dispatch::MethodDispatchError::InvalidPayload {
                                        method: method.to_owned(),
                                        source,
                                    })
                                    .map(|raw| {
                                        Self::$ExtVariant(impl_acp_from_method_and_params!(@wrap $ext_wrap ($ExtCtor(
                                            ::std::sync::Arc::from(method),
                                            ::std::sync::Arc::from(raw),
                                        ))))
                                    });
                            }
                        )?
                        Err($crate::dispatch::MethodDispatchError::UnknownMethod {
                            method: method.to_owned(),
                        })
                    }
                }
            }
        }
    };
    (@wrap plain $payload:expr) => {
        $payload
    };
    (@wrap boxed $payload:expr) => {
        Box::new($payload)
    };
}

macro_rules! impl_acp_from_method_and_result {
    (
        enum = $Enum:ty,
        ext = { $ExtVariant:ident, $ext_wrap:ident, $ExtCtor:path },
        arms = { $( $(#[$meta:meta])* $Variant:ident ($wrap:ident) => $method:expr ),* $(,)? }
    ) => {
        impl $Enum {
            /// Deserializes the `result` payload of a successful JSON-RPC
            /// response by dispatching on the method name of the originating
            /// request.
            ///
            /// Unlike the derived `Deserialize` implementation (which is
            /// `#[serde(untagged)]` and exists for schema generation), this
            /// never silently selects the wrong variant: unknown non-extension
            /// methods fail with [`MethodDispatchError::UnknownMethod`], and
            /// payloads that do not match the response's shape fail with
            /// [`MethodDispatchError::InvalidPayload`].
            ///
            /// # Errors
            ///
            /// Returns [`MethodDispatchError::UnknownMethod`] for method names
            /// that are not part of the protocol and do not start with the
            /// extension prefix `_`, and [`MethodDispatchError::InvalidPayload`]
            /// when the payload does not match the dispatched variant's shape.
            ///
            /// `result` is the optional `result` member of the JSON-RPC
            /// response; `None` and `null` are treated as an empty object.
            /// Extension methods (names starting with `_`) are routed to the
            /// enum's extension variant with the raw payload preserved.
            pub fn from_method_and_result(
                method: &str,
                result: Option<serde_json::Value>,
            ) -> ::std::result::Result<Self, $crate::dispatch::MethodDispatchError> {
                match method {
                    $(
                        $(#[$meta])*
                        m if m == $method => ::serde_json::from_value(
                            result.filter(|p| !p.is_null()).unwrap_or_else(|| ::serde_json::json!({})),
                        )
                        .map(|payload| Self::$Variant(impl_acp_from_method_and_result!(@wrap $wrap payload)))
                        .map_err(|source| $crate::dispatch::MethodDispatchError::InvalidPayload {
                            method: method.to_owned(),
                            source,
                        }),
                    )*
                    _ => {
                        if method.starts_with('_') {
                            let raw_result = result.unwrap_or(::serde_json::Value::Null);
                            return ::serde_json::value::RawValue::from_string(raw_result.to_string())
                                .map_err(|source| $crate::dispatch::MethodDispatchError::InvalidPayload {
                                    method: method.to_owned(),
                                    source,
                                })
                                .map(|raw| {
                                    Self::$ExtVariant(impl_acp_from_method_and_result!(@wrap $ext_wrap ($ExtCtor(
                                        ::std::sync::Arc::from(raw),
                                    ))))
                                });
                        }
                        Err($crate::dispatch::MethodDispatchError::UnknownMethod {
                            method: method.to_owned(),
                        })
                    }
                }
            }
        }
    };
    (@wrap plain $payload:expr) => {
        $payload
    };
    (@wrap boxed $payload:expr) => {
        Box::new($payload)
    };
}

impl_acp_from_method_and_params! {
    enum = crate::v1::ClientRequest,
    ext = { ExtMethodRequest, plain, crate::v1::ExtRequest::new },
    arms = {
        InitializeRequest (plain) => crate::v1::AGENT_METHOD_NAMES.initialize,
        AuthenticateRequest (plain) => crate::v1::AGENT_METHOD_NAMES.authenticate,
        #[cfg(feature = "unstable_llm_providers")]
        ListProvidersRequest (plain) => crate::v1::AGENT_METHOD_NAMES.providers_list,
        #[cfg(feature = "unstable_llm_providers")]
        SetProviderRequest (plain) => crate::v1::AGENT_METHOD_NAMES.providers_set,
        #[cfg(feature = "unstable_llm_providers")]
        DisableProviderRequest (plain) => crate::v1::AGENT_METHOD_NAMES.providers_disable,
        LogoutRequest (plain) => crate::v1::AGENT_METHOD_NAMES.logout,
        #[cfg(feature = "unstable_auth_status")]
        AuthStatusRequest (plain) => crate::v1::AGENT_METHOD_NAMES.auth_status,
        NewSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_new,
        LoadSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_load,
        ListSessionsRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_list,
        DeleteSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_delete,
        #[cfg(feature = "unstable_session_fork")]
        ForkSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_fork,
        #[cfg(feature = "unstable_session_archive")]
        ArchiveSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_archive,
        #[cfg(feature = "unstable_session_archive")]
        UnarchiveSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_unarchive,
        #[cfg(feature = "unstable_session_status")]
        SessionStatusRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_status,
        #[cfg(feature = "unstable_session_set_title")]
        SetSessionTitleRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_set_title,
        ResumeSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_resume,
        CloseSessionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_close,
        SetSessionModeRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_set_mode,
        SetSessionConfigOptionRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_set_config_option,
        PromptRequest (plain) => crate::v1::AGENT_METHOD_NAMES.session_prompt,
    }
}

impl_acp_from_method_and_params! {
    enum = crate::v1::AgentRequest,
    ext = { ExtMethodRequest, plain, crate::v1::ExtRequest::new },
    arms = {
        WriteTextFileRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.fs_write_text_file,
        ReadTextFileRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.fs_read_text_file,
        RequestPermissionRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.session_request_permission,
        CreateTerminalRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_create,
        TerminalOutputRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_output,
        ReleaseTerminalRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_release,
        WaitForTerminalExitRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_wait_for_exit,
        KillTerminalRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_kill,
        CreateElicitationRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.elicitation_create,
        #[cfg(feature = "unstable_mcp_over_acp")]
        MessageMcpRequest (plain) => crate::v1::CLIENT_METHOD_NAMES.mcp_message,
    }
}

impl_acp_from_method_and_params! {
    enum = crate::v1::ClientNotification,
    ext = { ExtNotification, plain, crate::v1::ExtNotification::new },
    arms = {
        CancelNotification (plain) => crate::v1::AGENT_METHOD_NAMES.session_cancel,
        #[cfg(feature = "unstable_mcp_over_acp")]
        MessageMcpNotification (plain) => crate::v1::AGENT_METHOD_NAMES.mcp_message,
    }
}

impl_acp_from_method_and_params! {
    enum = crate::v1::AgentNotification,
    ext = { ExtNotification, plain, crate::v1::ExtNotification::new },
    arms = {
        SessionNotification (plain) => crate::v1::CLIENT_METHOD_NAMES.session_update,
        CompleteElicitationNotification (plain) => crate::v1::CLIENT_METHOD_NAMES.elicitation_complete,
        #[cfg(feature = "unstable_auth_status")]
        AuthStatusUpdateNotification (plain) => crate::v1::CLIENT_METHOD_NAMES.auth_status_update,
    }
}

impl_acp_from_method_and_params! {
    enum = crate::v1::ProtocolLevelNotification,
    arms = {
        CancelRequestNotification (plain) => crate::v1::PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
    }
}

impl_acp_from_method_and_result! {
    enum = crate::v1::AgentResponse,
    ext = { ExtMethodResponse, plain, crate::v1::ExtResponse::new },
    arms = {
        InitializeResponse (plain) => crate::v1::AGENT_METHOD_NAMES.initialize,
        AuthenticateResponse (plain) => crate::v1::AGENT_METHOD_NAMES.authenticate,
        #[cfg(feature = "unstable_llm_providers")]
        ListProvidersResponse (plain) => crate::v1::AGENT_METHOD_NAMES.providers_list,
        #[cfg(feature = "unstable_llm_providers")]
        SetProviderResponse (plain) => crate::v1::AGENT_METHOD_NAMES.providers_set,
        #[cfg(feature = "unstable_llm_providers")]
        DisableProviderResponse (plain) => crate::v1::AGENT_METHOD_NAMES.providers_disable,
        LogoutResponse (plain) => crate::v1::AGENT_METHOD_NAMES.logout,
        #[cfg(feature = "unstable_auth_status")]
        AuthStatusResponse (plain) => crate::v1::AGENT_METHOD_NAMES.auth_status,
        NewSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_new,
        LoadSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_load,
        ListSessionsResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_list,
        DeleteSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_delete,
        #[cfg(feature = "unstable_session_fork")]
        ForkSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_fork,
        #[cfg(feature = "unstable_session_archive")]
        ArchiveSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_archive,
        #[cfg(feature = "unstable_session_archive")]
        UnarchiveSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_unarchive,
        #[cfg(feature = "unstable_session_status")]
        SessionStatusResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_status,
        #[cfg(feature = "unstable_session_set_title")]
        SetSessionTitleResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_set_title,
        ResumeSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_resume,
        CloseSessionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_close,
        SetSessionModeResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_set_mode,
        SetSessionConfigOptionResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_set_config_option,
        PromptResponse (plain) => crate::v1::AGENT_METHOD_NAMES.session_prompt,
    }
}

impl_acp_from_method_and_result! {
    enum = crate::v1::ClientResponse,
    ext = { ExtMethodResponse, plain, crate::v1::ExtResponse::new },
    arms = {
        WriteTextFileResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.fs_write_text_file,
        ReadTextFileResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.fs_read_text_file,
        RequestPermissionResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.session_request_permission,
        CreateTerminalResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_create,
        TerminalOutputResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_output,
        ReleaseTerminalResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_release,
        WaitForTerminalExitResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_wait_for_exit,
        KillTerminalResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.terminal_kill,
        CreateElicitationResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.elicitation_create,
        #[cfg(feature = "unstable_mcp_over_acp")]
        MessageMcpResponse (plain) => crate::v1::CLIENT_METHOD_NAMES.mcp_message,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_params! {
    enum = crate::v2::ClientRequest,
    ext = { ExtMethodRequest, boxed, crate::v2::ExtRequest::new },
    arms = {
        InitializeRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.initialize,
        LoginAuthRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.auth_login,
        #[cfg(feature = "unstable_llm_providers")]
        ListProvidersRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.providers_list,
        #[cfg(feature = "unstable_llm_providers")]
        SetProviderRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.providers_set,
        #[cfg(feature = "unstable_llm_providers")]
        DisableProviderRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.providers_disable,
        LogoutAuthRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.auth_logout,
        #[cfg(feature = "unstable_auth_status")]
        AuthStatusRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.auth_status,
        NewSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_new,
        ListSessionsRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_list,
        DeleteSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_delete,
        #[cfg(feature = "unstable_session_fork")]
        ForkSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_fork,
        #[cfg(feature = "unstable_session_archive")]
        ArchiveSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_archive,
        #[cfg(feature = "unstable_session_archive")]
        UnarchiveSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_unarchive,
        #[cfg(feature = "unstable_session_status")]
        SessionStatusRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_status,
        #[cfg(feature = "unstable_session_set_title")]
        SetSessionTitleRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_set_title,
        ResumeSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_resume,
        CloseSessionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_close,
        CancelToolCallRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.tool_call_cancel,
        CancelTerminalRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.terminal_cancel,
        SetSessionConfigOptionRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_set_config_option,
        PromptRequest (boxed) => crate::v2::AGENT_METHOD_NAMES.session_prompt,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_params! {
    enum = crate::v2::AgentRequest,
    ext = { ExtMethodRequest, boxed, crate::v2::ExtRequest::new },
    arms = {
        RequestPermissionRequest (boxed) => crate::v2::CLIENT_METHOD_NAMES.session_request_permission,
        CreateElicitationRequest (boxed) => crate::v2::CLIENT_METHOD_NAMES.elicitation_create,
        #[cfg(feature = "unstable_mcp_over_acp")]
        MessageMcpRequest (boxed) => crate::v2::CLIENT_METHOD_NAMES.mcp_message,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_params! {
    enum = crate::v2::ClientNotification,
    ext = { ExtNotification, boxed, crate::v2::ExtNotification::new },
    arms = {
        CancelSessionNotification (boxed) => crate::v2::AGENT_METHOD_NAMES.session_cancel,
        #[cfg(feature = "unstable_mcp_over_acp")]
        MessageMcpNotification (boxed) => crate::v2::AGENT_METHOD_NAMES.mcp_message,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_params! {
    enum = crate::v2::AgentNotification,
    ext = { ExtNotification, boxed, crate::v2::ExtNotification::new },
    arms = {
        UpdateSessionNotification (boxed) => crate::v2::CLIENT_METHOD_NAMES.session_update,
        CompleteElicitationNotification (boxed) => crate::v2::CLIENT_METHOD_NAMES.elicitation_complete,
        #[cfg(feature = "unstable_auth_status")]
        AuthStatusUpdateNotification (boxed) => crate::v2::CLIENT_METHOD_NAMES.auth_status_update,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_params! {
    enum = crate::v2::ProtocolLevelNotification,
    arms = {
        CancelRequestNotification (plain) => crate::v2::PROTOCOL_LEVEL_METHOD_NAMES.cancel_request,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_result! {
    enum = crate::v2::AgentResponse,
    ext = { ExtMethodResponse, boxed, crate::v2::ExtResponse::new },
    arms = {
        InitializeResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.initialize,
        LoginAuthResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.auth_login,
        #[cfg(feature = "unstable_llm_providers")]
        ListProvidersResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.providers_list,
        #[cfg(feature = "unstable_llm_providers")]
        SetProviderResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.providers_set,
        #[cfg(feature = "unstable_llm_providers")]
        DisableProviderResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.providers_disable,
        LogoutAuthResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.auth_logout,
        #[cfg(feature = "unstable_auth_status")]
        AuthStatusResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.auth_status,
        NewSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_new,
        ListSessionsResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_list,
        DeleteSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_delete,
        #[cfg(feature = "unstable_session_fork")]
        ForkSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_fork,
        #[cfg(feature = "unstable_session_archive")]
        ArchiveSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_archive,
        #[cfg(feature = "unstable_session_archive")]
        UnarchiveSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_unarchive,
        #[cfg(feature = "unstable_session_status")]
        SessionStatusResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_status,
        #[cfg(feature = "unstable_session_set_title")]
        SetSessionTitleResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_set_title,
        ResumeSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_resume,
        CloseSessionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_close,
        CancelToolCallResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.tool_call_cancel,
        CancelTerminalResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.terminal_cancel,
        SetSessionConfigOptionResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_set_config_option,
        PromptResponse (boxed) => crate::v2::AGENT_METHOD_NAMES.session_prompt,
    }
}

#[cfg(feature = "unstable_protocol_v2")]
impl_acp_from_method_and_result! {
    enum = crate::v2::ClientResponse,
    ext = { ExtMethodResponse, boxed, crate::v2::ExtResponse::new },
    arms = {
        RequestPermissionResponse (boxed) => crate::v2::CLIENT_METHOD_NAMES.session_request_permission,
        CreateElicitationResponse (boxed) => crate::v2::CLIENT_METHOD_NAMES.elicitation_create,
        #[cfg(feature = "unstable_mcp_over_acp")]
        MessageMcpResponse (boxed) => crate::v2::CLIENT_METHOD_NAMES.mcp_message,
    }
}
