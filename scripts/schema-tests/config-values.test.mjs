import assert from "node:assert/strict";
import test from "node:test";
import { schemaFiles, validators } from "./publications.mjs";

for (const version of ["v1", "v2"]) {
  for (const file of schemaFiles) {
    const { at } = validators(version, file);
    const validate = at("/$defs/SetSessionConfigOptionRequest");
    // Only the v2 root dispatches params by method.
    const root = version === "v2" ? at() : null;
    const params = { sessionId: "s", configId: "fast", type: "boolean" };

    test(`${version}/${file}: boolean config setters require boolean values`, () => {
      for (const value of [true, false]) {
        assert.equal(validate({ ...params, value }), true);
      }
      for (const value of ["false", "true", "", 0, null, [], {}]) {
        const request = { ...params, value };
        assert.equal(validate(request), false, JSON.stringify(request));
        if (root) {
          assert.equal(
            root({
              jsonrpc: "2.0",
              id: 1,
              method: "session/set_config_option",
              params: request,
            }),
            false,
            JSON.stringify(request),
          );
        }
      }
    });

    test(`${version}/${file}: string IDs and unknown string value types remain valid`, () => {
      const types = ["id", "text", ...(version === "v1" ? [undefined] : [])];
      for (const type of types) {
        assert.equal(
          validate({ sessionId: "s", configId: "model", type, value: "false" }),
          true,
        );
      }
    });

    if (file === "schema.unstable.json") {
      test(`${version}/${file}: initial config values use the same boolean constraint`, () => {
        const initial = at("/$defs/NewSessionRequest");
        const request = (value) => ({
          cwd: "/home/user/project",
          mcpServers: [],
          configOptions: [{ configId: "fast", type: "boolean", value }],
        });
        for (const value of [true, false]) {
          assert.equal(initial(request(value)), true);
        }
        assert.equal(initial(request("false")), false);
      });
    }
  }
}
