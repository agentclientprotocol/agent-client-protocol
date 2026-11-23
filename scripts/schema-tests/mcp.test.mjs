import assert from "node:assert/strict";
import test from "node:test";
import { validators } from "./publications.mjs";

for (const version of ["v1", "v2"]) {
  const { at } = validators(version, "schema.unstable.json");
  const validate = at("/$defs/MessageMcpRequest");
  const check = (attribution) =>
    validate({
      serverId: "s",
      requestId: "r",
      method: "tools/call",
      ...attribution,
    });
  test(`${version}: mcp/message attribution is optional`, () => {
    for (const attribution of [
      {},
      { sessionId: null, toolCallId: null },
      { sessionId: "s" },
      { sessionId: "s", toolCallId: "t" },
    ]) {
      assert.equal(check(attribution), true, JSON.stringify(attribution));
    }
  });
  test(`${version}: an mcp/message tool call needs a session`, () => {
    for (const attribution of [
      { toolCallId: "t" },
      { sessionId: null, toolCallId: "t" },
    ]) {
      assert.equal(check(attribution), false, JSON.stringify(attribution));
    }
  });
}
