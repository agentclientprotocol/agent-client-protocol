import assert from "node:assert/strict";
import test from "node:test";
import { validators } from "./publications.mjs";

for (const version of ["v1", "v2"]) {
  const { at } = validators(version, "schema.unstable.json");
  const setupRequests = [
    "NewSessionRequest",
    "ResumeSessionRequest",
    "ForkSessionRequest",
    ...(version === "v1" ? ["LoadSessionRequest"] : []),
  ];
  const cases = [
    ["SessionOptionsRequest", "additionalDirectories", ["/home/user/shared"]],
    ...setupRequests.map((name) => [
      name,
      "configOptions",
      [{ configId: "fast", type: "boolean", value: true }],
    ]),
  ];

  for (const [name, field, entries] of cases) {
    test(`${version}: ${name}.${field} accepts null without weakening items`, () => {
      const validate = at(`/$defs/${name}`);
      const params = {
        cwd: "/home/user/project",
        sessionId: "s",
        mcpServers: [],
      };
      assert.equal(validate(params), true);
      for (const value of [[], entries, null]) {
        assert.equal(
          validate({ ...params, [field]: value }),
          true,
          JSON.stringify({ field, value, errors: validate.errors }),
        );
      }
      for (const value of [
        {},
        false,
        "invalid",
        [null],
        [42],
        [{}],
        [{ configId: "fast", type: "boolean", value: null }],
      ]) {
        assert.equal(
          validate({ ...params, [field]: value }),
          false,
          JSON.stringify({ field, value }),
        );
      }
    });
  }
}
