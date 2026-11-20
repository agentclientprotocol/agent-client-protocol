import assert from "node:assert/strict";
import test from "node:test";
import { schemaFiles, validators } from "./publications.mjs";

for (const file of schemaFiles) {
  const { at } = validators("v2", file);
  const validate = at("/$defs/CreateElicitationRequest");
  const form = {
    mode: "form",
    requestedSchema: { type: "object", properties: {} },
  };
  test(`${file}: elicitation scope is selected by key presence`, () => {
    for (const mode of [
      form,
      { mode: "url", url: "https://example.org", elicitationId: "e" },
      { mode: "_custom" },
    ]) {
      const check = (scope) =>
        validate({ ...mode, message: "Choose", ...scope });
      for (const scope of [
        { sessionId: "s", toolCallId: null },
        { sessionId: "s", toolCallId: "t" },
        { requestId: 0, futureField: true },
      ]) {
        assert.equal(check(scope), true, JSON.stringify(scope));
      }
      for (const scope of [
        {},
        { sessionId: "s", requestId: 1 },
        { sessionId: "s", requestId: null },
        { sessionId: null, requestId: 1 },
        { requestId: 1, toolCallId: "t" },
        { requestId: 1, toolCallId: null },
      ]) {
        assert.equal(check(scope), false, JSON.stringify(scope));
      }
    }
  });
  test(`${file}: malformed typed items cannot match titled items`, () => {
    const choices = { anyOf: [{ const: "red", title: "Red" }] };
    const check = (items) =>
      validate({
        mode: "form",
        sessionId: "s",
        message: "Choose",
        requestedSchema: {
          type: "object",
          properties: { colors: { type: "array", items } },
        },
      });
    for (const items of [
      choices,
      { type: "string", enum: ["red"] },
      { ...choices, type: "_custom" },
      { ...choices, type: "future" },
    ]) {
      assert.equal(check(items), true, JSON.stringify(items));
    }
    for (const items of [
      { ...choices, type: "string" },
      { ...choices, type: "string", enum: 7 },
      { ...choices, type: null },
      { ...choices, type: 42 },
    ]) {
      assert.equal(check(items), false, JSON.stringify(items));
    }
  });
}
