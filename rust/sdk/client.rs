use std::{rc::Rc, sync::Arc};

use serde_json::value::RawValue;

use crate::{
    CreateTerminalRequest, CreateTerminalResponse, Error, ExtNotification, ExtRequest, ExtResponse,
    KillTerminalCommandRequest, KillTerminalCommandResponse, ReadTextFileRequest,
    ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionRequest, RequestPermissionResponse, SessionNotification,
    TerminalOutputRequest, TerminalOutputResponse, WaitForTerminalExitRequest,
    WaitForTerminalExitResponse, WriteTextFileRequest, WriteTextFileResponse,
};

/// Defines the interface that ACP-compliant clients must implement.
///
/// Clients are typically code editors (IDEs, text editors) that provide the interface
/// between users and AI agents. They manage the environment, handle user interactions,
/// and control access to resources.
#[async_trait::async_trait(?Send)]
pub trait Client {
    /// Requests permission from the user for a tool call operation.
    ///
    /// Called by the agent when it needs user authorization before executing
    /// a potentially sensitive operation. The client should present the options
    /// to the user and return their decision.
    ///
    /// If the client cancels the prompt turn via `session/cancel`, it MUST
    /// respond to this request with `RequestPermissionOutcome::Cancelled`.
    ///
    /// See protocol docs: [Requesting Permission](https://agentclientprotocol.com/protocol/tool-calls#requesting-permission)
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse, Error>;

    /// Handles session update notifications from the agent.
    ///
    /// This is a notification endpoint (no response expected) that receives
    /// real-time updates about session progress, including message chunks,
    /// tool calls, and execution plans.
    ///
    /// Note: Clients SHOULD continue accepting tool call updates even after
    /// sending a `session/cancel` notification, as the agent may send final
    /// updates before responding with the cancelled stop reason.
    ///
    /// See protocol docs: [Agent Reports Output](https://agentclientprotocol.com/protocol/prompt-turn#3-agent-reports-output)
    async fn session_notification(&self, args: SessionNotification) -> Result<(), Error>;

    /// Writes content to a text file in the client's file system.
    ///
    /// Only available if the client advertises the `fs.writeTextFile` capability.
    /// Allows the agent to create or modify files within the client's environment.
    ///
    /// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
    async fn write_text_file(
        &self,
        _args: WriteTextFileRequest,
    ) -> Result<WriteTextFileResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Reads content from a text file in the client's file system.
    ///
    /// Only available if the client advertises the `fs.readTextFile` capability.
    /// Allows the agent to access file contents within the client's environment.
    ///
    /// See protocol docs: [Client](https://agentclientprotocol.com/protocol/overview#client)
    async fn read_text_file(
        &self,
        _args: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Executes a command in a new terminal
    ///
    /// Only available if the `terminal` Client capability is set to `true`.
    ///
    /// Returns a `TerminalId` that can be used with other terminal methods
    /// to get the current output, wait for exit, and kill the command.
    ///
    /// The `TerminalId` can also be used to embed the terminal in a tool call
    /// by using the `ToolCallContent::Terminal` variant.
    ///
    /// The Agent is responsible for releasing the terminal by using the `terminal/release`
    /// method.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    async fn create_terminal(
        &self,
        _args: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Gets the terminal output and exit status
    ///
    /// Returns the current content in the terminal without waiting for the command to exit.
    /// If the command has already exited, the exit status is included.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    async fn terminal_output(
        &self,
        _args: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Releases a terminal
    ///
    /// The command is killed if it hasn't exited yet. Use `terminal/wait_for_exit`
    /// to wait for the command to exit before releasing the terminal.
    ///
    /// After release, the `TerminalId` can no longer be used with other `terminal/*` methods,
    /// but tool calls that already contain it, continue to display its output.
    ///
    /// The `terminal/kill` method can be used to terminate the command without releasing
    /// the terminal, allowing the Agent to call `terminal/output` and other methods.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    async fn release_terminal(
        &self,
        _args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Waits for the terminal command to exit and return its exit status
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    async fn wait_for_terminal_exit(
        &self,
        _args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Kills the terminal command without releasing the terminal
    ///
    /// While `terminal/release` will also kill the command, this method will keep
    /// the `TerminalId` valid so it can be used with other methods.
    ///
    /// This method can be helpful when implementing command timeouts which terminate
    /// the command as soon as elapsed, and then get the final output so it can be sent
    /// to the model.
    ///
    /// Note: `terminal/release` when `TerminalId` is no longer needed.
    ///
    /// See protocol docs: [Terminals](https://agentclientprotocol.com/protocol/terminals)
    async fn kill_terminal_command(
        &self,
        _args: KillTerminalCommandRequest,
    ) -> Result<KillTerminalCommandResponse, Error> {
        Err(Error::method_not_found())
    }

    /// Handles extension method requests from the agent.
    ///
    /// Allows the Agent to send an arbitrary request that is not part of the ACP spec.
    /// Extension methods provide a way to add custom functionality while maintaining
    /// protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    async fn ext_method(&self, _args: ExtRequest) -> Result<ExtResponse, Error> {
        Ok(RawValue::NULL.to_owned().into())
    }

    /// Handles extension notifications from the agent.
    ///
    /// Allows the Agent to send an arbitrary notification that is not part of the ACP spec.
    /// Extension notifications provide a way to send one-way messages for custom functionality
    /// while maintaining protocol compatibility.
    ///
    /// See protocol docs: [Extensibility](https://agentclientprotocol.com/protocol/extensibility)
    async fn ext_notification(&self, _args: ExtNotification) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl<T: Client> Client for Rc<T> {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse, Error> {
        self.as_ref().request_permission(args).await
    }
    async fn write_text_file(
        &self,
        args: WriteTextFileRequest,
    ) -> Result<WriteTextFileResponse, Error> {
        self.as_ref().write_text_file(args).await
    }
    async fn read_text_file(
        &self,
        args: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse, Error> {
        self.as_ref().read_text_file(args).await
    }
    async fn session_notification(&self, args: SessionNotification) -> Result<(), Error> {
        self.as_ref().session_notification(args).await
    }
    async fn create_terminal(
        &self,
        args: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse, Error> {
        self.as_ref().create_terminal(args).await
    }
    async fn terminal_output(
        &self,
        args: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse, Error> {
        self.as_ref().terminal_output(args).await
    }
    async fn release_terminal(
        &self,
        args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse, Error> {
        self.as_ref().release_terminal(args).await
    }
    async fn wait_for_terminal_exit(
        &self,
        args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse, Error> {
        self.as_ref().wait_for_terminal_exit(args).await
    }
    async fn kill_terminal_command(
        &self,
        args: KillTerminalCommandRequest,
    ) -> Result<KillTerminalCommandResponse, Error> {
        self.as_ref().kill_terminal_command(args).await
    }
    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        self.as_ref().ext_method(args).await
    }
    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.as_ref().ext_notification(args).await
    }
}

#[async_trait::async_trait(?Send)]
impl<T: Client> Client for Arc<T> {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> Result<RequestPermissionResponse, Error> {
        self.as_ref().request_permission(args).await
    }
    async fn write_text_file(
        &self,
        args: WriteTextFileRequest,
    ) -> Result<WriteTextFileResponse, Error> {
        self.as_ref().write_text_file(args).await
    }
    async fn read_text_file(
        &self,
        args: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse, Error> {
        self.as_ref().read_text_file(args).await
    }
    async fn session_notification(&self, args: SessionNotification) -> Result<(), Error> {
        self.as_ref().session_notification(args).await
    }
    async fn create_terminal(
        &self,
        args: CreateTerminalRequest,
    ) -> Result<CreateTerminalResponse, Error> {
        self.as_ref().create_terminal(args).await
    }
    async fn terminal_output(
        &self,
        args: TerminalOutputRequest,
    ) -> Result<TerminalOutputResponse, Error> {
        self.as_ref().terminal_output(args).await
    }
    async fn release_terminal(
        &self,
        args: ReleaseTerminalRequest,
    ) -> Result<ReleaseTerminalResponse, Error> {
        self.as_ref().release_terminal(args).await
    }
    async fn wait_for_terminal_exit(
        &self,
        args: WaitForTerminalExitRequest,
    ) -> Result<WaitForTerminalExitResponse, Error> {
        self.as_ref().wait_for_terminal_exit(args).await
    }
    async fn kill_terminal_command(
        &self,
        args: KillTerminalCommandRequest,
    ) -> Result<KillTerminalCommandResponse, Error> {
        self.as_ref().kill_terminal_command(args).await
    }
    async fn ext_method(&self, args: ExtRequest) -> Result<ExtResponse, Error> {
        self.as_ref().ext_method(args).await
    }
    async fn ext_notification(&self, args: ExtNotification) -> Result<(), Error> {
        self.as_ref().ext_notification(args).await
    }
}
