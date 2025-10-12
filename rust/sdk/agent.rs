use std::{rc::Rc, sync::Arc};

use serde_json::value::RawValue;

use crate::{
    AuthenticateRequest, AuthenticateResponse, CancelNotification, Error, ExtNotification,
    ExtRequest, ExtResponse, InitializeRequest, InitializeResponse, LoadSessionRequest,
    LoadSessionResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse,
    SetSessionModeRequest, SetSessionModeResponse,
};
#[cfg(feature = "unstable")]
use crate::{SetSessionModelRequest, SetSessionModelResponse};

/// Defines the interface that all ACP-compliant agents must implement.
///
/// Agents are programs that use generative AI to autonomously modify code. They handle
/// requests from clients and execute tasks using language models and tools.
#[async_trait::async_trait(?Send)]
pub trait Agent {
    /// Establishes the connection with a client and negotiates protocol capabilities.
    ///
    /// This method is called once at the beginning of the connection to:
    /// - Negotiate the protocol version to use
    /// - Exchange capability information between client and agent
    /// - Determine available authentication methods
    ///
    /// The agent should respond with its supported protocol version and capabilities.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    async fn initialize(&self, args: InitializeRequest) -> Result<InitializeResponse, Error>;

    /// Authenticates the client using the specified authentication method.
    ///
    /// Called when the agent requires authentication before allowing session creation.
    /// The client provides the authentication method ID that was advertised during initialization.
    ///
    /// After successful authentication, the client can proceed to create sessions with
    /// `new_session` without receiving an `auth_required` error.
    ///
    /// See protocol docs: [Initialization](https://agentclientprotocol.com/protocol/initialization)
    async fn authenticate(&self, args: AuthenticateRequest) -> Result<AuthenticateResponse, Error>;

    /// Creates a new conversation session with the agent.
    ///
    /// Sessions represent independent conversation contexts with their own history and state.
    ///
    /// The agent should:
    /// - Create a new session context
    /// - Connect to any specified MCP servers
    /// - Return a unique session ID for future requests
    ///
    /// May return an `auth_required` error if the agent requires authentication.
    ///
    /// See protocol docs: [Session Setup](https://agentclientprotocol.com/protocol/session-setup)
    async fn new_session(&self, args: NewSessionRequest) -> Result<NewSessionResponse, Error>;

    /// Processes a user prompt within a session.
    ///
    /// This method handles the whole lifecycle of a prompt:
    /// - Receives user messages with optional context (files, images, etc.)
    /// - Processes the prompt using language models
    /// - Reports language model content and tool calls to the Clients
    /// - Requests permission to run tools
    /// - Executes any requested tool calls
    /// - Returns when the turn is complete with a stop reason
    ///
    /// See protocol docs: [Prompt Turn](https://agentclientprotocol.com/protocol/prompt-turn)
    async fn prompt(&self, args: PromptRequest) -> Result<PromptResponse, Error>;

    /// Cancels ongoing operations for a session.
    ///
    /// This is a notification sent by the client to cancel an ongoing prompt turn.
    ///
    /// Upon receiving this notification, the Agent SHOULD:
    /// - Stop all language model requests as soon as possible
    /// - Abort all tool call invocations in progress
    /// - Send any pending `session/update` notifications
    /// - Respond to the original `session/prompt` request with `StopReason::Cancelled`
    ///
    /// See protocol docs: [Cancellation](https://agentclientprotocol.com/protocol/prompt-turn#cancellation)
    async fn cancel(&self, args: CancelNotification) -> Result<(), Error>;

    /// Loads an existing session to resume a previous conversation.
    ///
    /// This method is only available if the agent advertises the `loadSession` capability.
    ///
    /// The agent should:
    /// - Restore the session context and conversation history
    /// - Connect to the specified MCP servers
    /// - Stream the entire conversation history back to the client via notifications
    ///
    /// See protocol docs: [Loading Sessions](https://agentclientprotocol.com/protocol/session-setup#loading-sessions)
    async fn load_session(&self, _args: LoadSessionRequest) -> Result<LoadSessionResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Sets the current mode for a session.
    ///
    /// Allows switching between different agent modes (e.g., "ask", "architect", "code")
    /// that affect system prompts, tool availability, and permission behaviors.
    ///
    /// The mode must be one of the modes advertised in `availableModes` during session
    /// creation or loading. Agents may also change modes autonomously and notify the
    /// client via `current_mode_update` notifications.
    ///
    /// This method can be called at any time during a session, whether the Agent is
    /// idle or actively generating a response.
    ///
    /// See protocol docs: [Session Modes](https://agentclientprotocol.com/protocol/session-modes)
    async fn set_session_mode(
        &self,
        _args: SetSessionModeRequest,
    ) -> Result<SetSessionModeResponse, Error> {
        Err(Error::method_not_found())
    }

    /// **UNSTABLE**
    ///
    /// This capability is not part of the spec yet, and may be removed or changed at any point.
    ///
    /// Select a model for a given session.
    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        _args: SetSessionModelRequest,
    ) -> Result<SetSessionModelResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Handles extension method requests from the client.
    ///
    /// Extension methods provide a way to add custom functionality while maintaining
    /// protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    async fn ext_method(&self, _args: ExtRequest) -> Result<ExtResponse, Error> {
        Ok(RawValue::NULL.to_owned().into())
    }

    /// Handles extension notifications from the client.
    ///
    /// Extension notifications provide a way to send one-way messages for custom functionality
    /// while maintaining protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    async fn ext_notification(&self, _args: ExtNotification) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<T: Agent> Agent for Rc<T> {
    async fn initialize(&self, args: InitializeRequest) -> Result<InitializeResponse, Error> {
        self.as_ref().initialize(args).await
    }
    async fn authenticate(&self, args: AuthenticateRequest) -> Result<AuthenticateResponse, Error> {
        self.as_ref().authenticate(args).await
    }
    async fn new_session(&self, args: NewSessionRequest) -> Result<NewSessionResponse, Error> {
        self.as_ref().new_session(args).await
    }
    async fn load_session(&self, args: LoadSessionRequest) -> Result<LoadSessionResponse, Error> {
        self.as_ref().load_session(args).await
    }
    async fn set_session_mode(
        &self,
        args: SetSessionModeRequest,
    ) -> Result<SetSessionModeResponse, Error> {
        self.as_ref().set_session_mode(args).await
    }
    async fn prompt(&self, args: PromptRequest) -> Result<PromptResponse, Error> {
        self.as_ref().prompt(args).await
    }
    async fn cancel(&self, args: CancelNotification) -> Result<(), Error> {
        self.as_ref().cancel(args).await
    }
    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        args: SetSessionModelRequest,
    ) -> Result<SetSessionModelResponse, Error> {
        self.as_ref().set_session_model(args).await
    }
    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        self.as_ref().ext_method(args).await
    }
    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.as_ref().ext_notification(args).await
    }
}

#[async_trait::async_trait(?Send)]
impl<T: Agent> Agent for Arc<T> {
    async fn initialize(&self, args: InitializeRequest) -> Result<InitializeResponse, Error> {
        self.as_ref().initialize(args).await
    }
    async fn authenticate(&self, args: AuthenticateRequest) -> Result<AuthenticateResponse, Error> {
        self.as_ref().authenticate(args).await
    }
    async fn new_session(&self, args: NewSessionRequest) -> Result<NewSessionResponse, Error> {
        self.as_ref().new_session(args).await
    }
    async fn load_session(&self, args: LoadSessionRequest) -> Result<LoadSessionResponse, Error> {
        self.as_ref().load_session(args).await
    }
    async fn set_session_mode(
        &self,
        args: SetSessionModeRequest,
    ) -> Result<SetSessionModeResponse, Error> {
        self.as_ref().set_session_mode(args).await
    }
    async fn prompt(&self, args: PromptRequest) -> Result<PromptResponse, Error> {
        self.as_ref().prompt(args).await
    }
    async fn cancel(&self, args: CancelNotification) -> Result<(), Error> {
        self.as_ref().cancel(args).await
    }
    #[cfg(feature = "unstable")]
    async fn set_session_model(
        &self,
        args: SetSessionModelRequest,
    ) -> Result<SetSessionModelResponse, Error> {
        self.as_ref().set_session_model(args).await
    }
    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        self.as_ref().ext_method(args).await
    }
    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.as_ref().ext_notification(args).await
    }
}
