import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { schemaFiles, validators } from "./publications.mjs";

const frame = (members) => ({ jsonrpc: "2.0", ...members });
const prompt = { sessionId: "s", prompt: [{ type: "text", text: "Hello" }] };
const cancel = { method: "session/cancel", params: { sessionId: "s" } };
const protocol = { method: "$/cancel_request", params: { requestId: 0 } };

for (const file of schemaFiles) {
  const { at, schema } = validators("v2", file);
  const root = at();
  const envelope = at("rpcEnvelope");
  const client = at("clientToAgent");
  const agent = at("agentToClient");
  const index = schema["x-methods"];
  const check = (v, input, want) =>
    assert.equal(v(input), want, JSON.stringify(input));

  test(`${file}: envelope kinds are exclusive`, () => {
    for (const bad of [
      [],
      { ...cancel },
      { jsonrpc: "1.0", ...cancel },
      frame({}),
      frame({ ...cancel, id: null }),
      frame({ ...cancel, result: {} }),
      frame({ ...cancel, error: { code: -32603, message: "e" } }),
      frame({ id: null, result: {} }),
      frame({ result: {} }),
      frame({ id: true, result: {} }),
      frame({ id: 1, result: {}, error: { code: -32603, message: "e" } }),
      frame({ id: 1, error: null }),
      frame({ id: 1, error: { code: "bad", message: "e" } }),
    ]) {
      check(envelope, bad, false);
      check(root, bad, false);
    }
    for (const good of [
      frame(cancel),
      frame(protocol),
      frame({ id: null, error: { code: -32700, message: "Parse error" } }),
      frame({ id: 1, result: null }),
    ]) {
      check(envelope, good, true);
      check(root, good, true);
    }
  });

  test(`${file}: route by sender, method and presence of id, not params shape`, () => {
    const call = frame({
      id: 0,
      method: "session/prompt",
      params: { ...prompt, futureField: true },
      futureEnvelopeField: true,
    });
    check(root, call, true);
    check(client, call, true);
    check(agent, call, false);
    for (const bad of [
      frame({ id: 1, method: "session/prompt" }),
      frame({ id: 1, method: "session/prompt", params: {} }),
      frame({
        id: 1,
        method: "session/prompt",
        params: { sessionId: "s", prompt: [] },
      }),
      frame({ method: "session/prompt", params: prompt }),
      frame({ id: 1, ...cancel }),
      frame({ id: 1, ...protocol }),
      frame({ ...protocol, params: { requestId: null } }),
      frame({
        method: "session/update",
        params: {
          sessionId: "s",
          update: { sessionUpdate: "agent_message", messageId: 1 },
        },
      }),
      frame({
        id: 2,
        method: "elicitation/create",
        params: {
          mode: "form",
          message: "Choose",
          sessionId: "s",
          requestId: null,
          requestedSchema: { type: "object", properties: {} },
        },
      }),
    ]) {
      check(envelope, bad, true);
      check(root, bad, false);
    }
    check(agent, frame(protocol), true);
    check(client, frame(protocol), true);
    const update = frame({
      method: "session/update",
      params: { sessionId: "s", update: { sessionUpdate: "_custom", key: 1 } },
    });
    check(agent, update, true);
    check(client, update, false);
    check(root, update, true);
    const unknown = frame({ id: "id", method: "future/operation", params: {} });
    check(envelope, unknown, true);
    check(root, unknown, false);
    // Negotiate first: an incoming request offering a later major version need
    // not conform to this version's initialization payload.
    const futureInit = frame({
      id: 0,
      method: "initialize",
      params: { protocolVersion: 3, info: "new shape" },
    });
    check(envelope, futureInit, true);
    check(root, futureInit, false);
  });

  test(`${file}: extension envelopes obey JSON-RPC without closing extension payloads`, () => {
    for (const fields of [{}, { id: "x" }]) {
      for (const params of [
        {},
        { params: [] },
        { params: { "vendor/field": [1, 2] } },
      ]) {
        const m = frame({ method: "_example", ...fields, ...params });
        for (const validate of [agent, client, root]) check(validate, m, true);
      }
      for (const params of [null, false, "value", 42])
        check(root, frame({ method: "_x", ...fields, params }), false);
    }
  });

  test(`${file}: check results with the originating request and errors with their code`, () => {
    const parse = at(index.agent["session/prompt"].result.slice(1));
    check(root, frame({ id: 7, result: {} }), true); // No method on this frame.
    check(parse, {}, false);
    check(parse, { messageId: "m" }, true);
    check(at(index.agent["auth/logout"].result.slice(1)), {}, true);
    const fail = (code, data) =>
      frame({
        id: 7,
        error: { code, message: "e", ...(data === undefined ? {} : { data }) },
      });
    for (const data of [
      undefined,
      null,
      {},
      { requested: 2 },
      { requested: null, supported: [2] },
    ]) {
      check(root, fail(-32022, data), false);
      check(at("/$defs/Error"), fail(-32022, data).error, false);
    }
    check(
      root,
      fail(-32022, { requested: 1, supported: [], future: true }),
      true,
    );
    check(root, fail(-32002, { uri: "resource:other" }), true); // Not necessarily a missing session.
    check(root, fail(999, ["opaque", 1]), true);
  });

  test(`${file}: every published method has a usable routing entry`, () => {
    const meta = JSON.parse(
      readFileSync(
        new URL(
          `../../schema/v2/${file.replace("schema", "meta")}`,
          import.meta.url,
        ),
      ),
    );
    const sides = { agent: client, client: agent, protocol: client };
    if (meta.proxyMethods) sides.proxy = at("proxyExchange");
    assert.deepEqual(Object.keys(index).sort(), Object.keys(sides).sort());
    for (const [side, validate] of Object.entries(sides)) {
      const routes = index[side];
      assert.deepEqual(
        Object.keys(routes).sort(),
        Object.values(meta[side + "Methods"]).sort(),
      );
      for (const [method, route] of Object.entries(routes)) {
        assert.equal("result" in route, "request" in route, method);
        for (const [kind, reference] of Object.entries(route)) {
          const pointer = reference.slice(1);
          const payload = at(pointer);
          const def = schema.$defs[reference.split("/").at(-1)];
          assert.equal(def["x-method"], method);
          assert.equal(def["x-side"], side);
          if (kind === "result") continue;
          const fields = kind === "request" ? { id: 1, method } : { method };
          // An omitted params is legal precisely for a method with no required arguments.
          for (const checkFrame of [root, validate]) {
            check(checkFrame, frame(fields), payload({}));
            check(checkFrame, frame({ ...fields, params: {} }), payload({}));
            check(checkFrame, frame({ ...fields, params: null }), false);
          }
        }
      }
    }
    assert.equal(
      index.agent["session/prompt"].request,
      "#/$defs/PromptRequest",
    );
    assert.equal(
      index.agent["session/prompt"].result,
      "#/$defs/PromptResponse",
    );
    assert.equal(
      index.client["session/update"].notification,
      "#/$defs/UpdateSessionNotification",
    );
    // A variant missing from dispatch must not be silently lost, even if a
    // different kind shares its name (such as proxy/successor).
    for (const [name, def] of Object.entries(schema.$defs)) {
      if (!def["x-method"]) continue;
      assert(
        Object.values(index[def["x-side"]][def["x-method"]]).includes(
          `#/$defs/${name}`,
        ),
        name,
      );
    }
  });

  test(`${file}: Draft-only routes do not leak into the baseline`, () => {
    const successor = frame({
      method: "proxy/successor",
      params: { method: "initialize" },
    });
    const mcp = frame({
      method: "mcp/message",
      params: { serverId: "s", requestId: "r", method: "tools/list" },
    });
    if (file === "schema.json") {
      assert.equal(index.proxy, undefined);
      check(root, successor, false);
      check(root, frame({ ...mcp, id: 1 }), false);
    } else {
      const proxy = at("proxyExchange");
      check(proxy, successor, true);
      check(proxy, { ...successor, id: 3 }, true);
      check(client, successor, false);
      check(proxy, frame(protocol), true);
      check(agent, { ...mcp, id: 1 }, true);
      check(client, { ...mcp, id: 1 }, false);
      check(client, mcp, true);
      check(agent, mcp, false);
    }
  });
}
