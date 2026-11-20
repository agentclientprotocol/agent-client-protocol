import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import test from "node:test";
import { createAjv, schemaFiles, validators } from "./publications.mjs";

test("new keywords and formats need an explicit registration", () => {
  for (const version of ["v1", "v2"]) {
    const ajv = createAjv(version);
    assert.throws(
      () => ajv.compile({ type: "object", "x-unregistered": true }),
      /unknown keyword: "x-unregistered"/,
    );
    assert.throws(
      () => ajv.compile({ type: "integer", format: "uint128" }),
      /unknown format "uint128"/,
    );
  }
});

for (const version of ["v1", "v2"]) {
  for (const file of schemaFiles) {
    const label = `${version}/${file}`;
    const { at, schema } = validators(version, file);

    test(`${label}: compile the root and each public definition`, (t) => {
      at();
      const names = Object.keys(schema.$defs);
      assert.ok(names.length > 0);
      // Ajv need not visit a definition that nothing in the root references.
      for (const name of names) at(`/$defs/${name}`);
      t.diagnostic(`${names.length} public definitions`);
    });

    test(`${label}: assert published formats and numeric bounds`, () => {
      const uri = at("/$defs/ElicitationUrlMode/properties/url");
      assert.equal(uri("urn:example:settings"), true);
      assert.equal(uri("relative/path"), false);

      if (version === "v2") {
        // The v1 timestamp does not declare a format.
        const timestamp = at("/$defs/SessionInfo/properties/updatedAt");
        assert.equal(timestamp("2024-02-29T12:00:00+05:30"), true);
        assert.equal(timestamp("2024-02-30T12:00:00Z"), false);
        assert.equal(timestamp("2024-02-29T12:00:00"), false);
      }

      const protocolVersion = at("/$defs/ProtocolVersion");
      for (const value of [0, 65535])
        assert.equal(protocolVersion(value), true);
      for (const value of [-1, 65536])
        assert.equal(protocolVersion(value), false);
    });

    test(`${label}: validate complete JSON-RPC examples in its guides`, (t) => {
      const guide = `docs/protocol/${version}/${file === "schema.unstable.json" ? "draft/" : ""}`;
      const dir = new URL(`../../${guide}`, import.meta.url);
      const validate = at();
      let checked = 0;
      for (const name of readdirSync(dir)) {
        // The migration guide intentionally shows both versions side by side.
        if (!name.endsWith(".mdx") || name === "migration.mdx") continue;
        const text = readFileSync(new URL(name, dir), "utf8");
        const fences =
          /^[ \t]*```json(?:[ \t][^\r\n]*)?\r?\n([\s\S]*?)^[ \t]*```[ \t]*\r?$/gm;
        for (const match of text.matchAll(fences)) {
          const line = text.slice(0, match.index).split("\n").length;
          const location = `${guide}${name}:${line}`;
          let example;
          try {
            example = JSON.parse(match[1]);
          } catch (cause) {
            throw new Error(`Invalid JSON in ${location}`, { cause });
          }
          if (!example || !Object.hasOwn(example, "jsonrpc")) continue; // Payload fragment.
          if (!validate(example))
            assert.fail(
              `Invalid JSON-RPC example in ${location}: ${JSON.stringify(validate.errors)}`,
            );
          checked++;
        }
      }
      assert.ok(checked > 0, `No JSON-RPC examples found under ${guide}`);
      t.diagnostic(`${checked} complete JSON-RPC examples`);
    });
  }
}
