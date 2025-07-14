import { describe, it, expect, beforeEach } from "vitest";
import {
  Agent,
  Client,
  Connection,
  InitializeParams,
  InitializeResponse,
  LATEST_PROTOCOL_VERSION,
  PushToolCallParams,
  PushToolCallResponse,
  ReadTextFileParams,
  ReadTextFileResponse,
  RequestToolCallConfirmationParams,
  RequestToolCallConfirmationResponse,
  SendUserMessageParams,
  StreamAssistantMessageChunkParams,
  UpdateToolCallParams,
  WriteTextFileParams,
} from "./acp.js";

describe("Connection", () => {
  let clientToAgent: TransformStream;
  let agentToClient: TransformStream;

  beforeEach(() => {
    clientToAgent = new TransformStream();
    agentToClient = new TransformStream();
  });

  it("handles errors in bidirectional communication", async () => {
    // Create client that throws errors
    class TestClient extends StubClient {
      async pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
        throw new Error("Tool call failed");
      }
    }

    // Create agent that throws errors
    class TestAgent extends StubAgent {
      async initialize(_: InitializeParams): Promise<InitializeResponse> {
        throw new Error("Failed to create thread");
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (agent) => new TestClient(agent),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Test error handling in client->agent direction
    await expect(
      clientConnection.pushToolCall({
        label: "/missing.ts",
        icon: "fileSearch",
      }),
    ).rejects.toThrow();

    // Test error handling in agent->client direction
    await expect(
      agentConnection.initialize({ protocolVersion: LATEST_PROTOCOL_VERSION }),
    ).rejects.toThrow();
  });

  it("handles concurrent requests", async () => {
    // Create client with delayed responses
    class TestClient extends StubClient {
      toolCall: number = 0;

      async pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
        this.toolCall++;
        const id = this.toolCall;
        console.log(id);
        await new Promise((resolve) => setTimeout(resolve, 40));
        console.log(id);
        return { id };
      }
    }

    // Create agent with delayed responses
    class TestAgent extends StubAgent {}

    Connection.clientToAgent(
      (a) => new TestClient(a),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send multiple concurrent requests
    const promises = [
      clientConnection.pushToolCall({
        label: "Tool Call 1",
        icon: "fileSearch",
      }),
      clientConnection.pushToolCall({
        label: "Tool Call 2",
        icon: "fileSearch",
      }),
      clientConnection.pushToolCall({
        label: "Tool Call 3",
        icon: "fileSearch",
      }),
    ];

    const results = await Promise.all(promises);

    // Verify all requests completed successfully
    expect(results[0]).toHaveProperty("id", 1);
    expect(results[1]).toHaveProperty("id", 2);
    expect(results[2]).toHaveProperty("id", 3);
  });

  it("handles message ordering correctly", async () => {
    const messageLog: string[] = [];

    class TestClient extends StubClient {
      async pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
        messageLog.push("pushToolCall called");
        return { id: 0 };
      }
      async updateToolCall(_: UpdateToolCallParams): Promise<void> {
        messageLog.push("updateToolCall called");
      }
    }

    class TestAgent extends StubAgent {
      async initialize(request: InitializeParams): Promise<InitializeResponse> {
        messageLog.push("initialize called");
        return {
          protocolVersion: request.protocolVersion,
          isAuthenticated: true,
        };
      }
    }

    // Set up connections
    const agentConnection = Connection.clientToAgent(
      (client) => new TestClient(client),
      clientToAgent.writable,
      agentToClient.readable,
    );

    const clientConnection = Connection.agentToClient(
      (client) => new TestAgent(client),
      agentToClient.writable,
      clientToAgent.readable,
    );

    // Send requests in specific order
    await agentConnection.initialize({
      protocolVersion: LATEST_PROTOCOL_VERSION,
    });
    let { id } = await clientConnection.pushToolCall({
      icon: "folder",
      label: "Folder",
    });
    await clientConnection.updateToolCall({
      content: {
        type: "markdown",
        markdown: "Markdown",
      },
      status: "finished",
      toolCallId: id,
    });

    // Verify order
    expect(messageLog).toEqual([
      "initialize called",
      "pushToolCall called",
      "updateToolCall called",
    ]);
  });
});

class StubAgent implements Agent {
  constructor(private client: Client) {}
  initialize(_: InitializeParams): Promise<InitializeResponse> {
    throw new Error("Method not implemented.");
  }
  authenticate(): Promise<void> {
    throw new Error("Method not implemented.");
  }
  sendUserMessage(_: SendUserMessageParams): Promise<void> {
    throw new Error("Method not implemented.");
  }
  cancelSendMessage(): Promise<void> {
    throw new Error("Method not implemented.");
  }
}

class StubClient implements Client {
  constructor(private agent: Agent) {}
  streamAssistantMessageChunk(
    _: StreamAssistantMessageChunkParams,
  ): Promise<void> {
    throw new Error("Method not implemented.");
  }
  requestToolCallConfirmation(
    _: RequestToolCallConfirmationParams,
  ): Promise<RequestToolCallConfirmationResponse> {
    throw new Error("Method not implemented.");
  }
  pushToolCall(_: PushToolCallParams): Promise<PushToolCallResponse> {
    throw new Error("Method not implemented.");
  }
  updateToolCall(_: UpdateToolCallParams): Promise<void> {
    throw new Error("Method not implemented.");
  }
  readTextFile(_: ReadTextFileParams): Promise<ReadTextFileResponse> {
    throw new Error("Method not implemented.");
  }
  writeTextFile(_: WriteTextFileParams): Promise<void> {
    throw new Error("Method not implemented.");
  }
}
