import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";

export const schemaFiles = ["schema.json", "schema.unstable.json"];

export function createAjv(version) {
  const ajv = new Ajv2020({ strict: true, validateFormats: true });
  // Annotations for documentation, SDK generators and lenient readers. List them
  // by name so that a typo or a new extension requires a deliberate change here.
  ajv.addVocabulary([
    "x-docs-ignore",
    "x-method",
    "x-side",
    "x-state",
    "x-scope",
    "x-deserialize-default-on-error",
    "x-deserialize-default-on-null",
    "x-deserialize-skip-invalid-items",
  ]);
  if (version === "v1") {
    ajv.addKeyword("discriminator"); // Codegen hint, not an assertion.
  } else {
    ajv.addKeyword("x-methods"); // Lookup for validators that know the originating method.
    // Ajv 8 resolves 2020-12 anchors, but strict schema compilation needs the name registered.
    ajv.addKeyword("$anchor");
  }

  addFormats(ajv, ["uri", "date-time"]);
  // Rust numeric type hints. Wire limits belong in minimum/maximum, not here.
  for (const name of ["int32", "int64", "uint16", "uint32", "uint64", "double"])
    ajv.addFormat(name, true);
  return ajv;
}

export function validators(version, file) {
  const url = new URL(`../../schema/${version}/${file}`, import.meta.url);
  const ajv = createAjv(version);
  const schema = JSON.parse(readFileSync(url, "utf8"));
  ajv.addSchema(schema, url.href);
  return {
    schema,
    at(fragment = "") {
      const ref = `${url.href}#${fragment}`;
      const validate = ajv.getSchema(ref);
      assert.ok(validate, `No schema at ${ref}`);
      return validate;
    },
  };
}
