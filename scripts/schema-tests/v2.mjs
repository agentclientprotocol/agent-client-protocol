import { readFileSync } from "node:fs";
import Ajv2020 from "ajv/dist/2020.js";

export const publications = ["schema.json", "schema.unstable.json"];

export function validators(file) {
  const url = new URL(`../../schema/v2/${file}`, import.meta.url);
  // The published schemas carry generator annotations and nonstandard format names.
  const ajv = new Ajv2020({ strict: false, validateFormats: false });
  const schema = JSON.parse(readFileSync(url, "utf8"));
  ajv.addSchema(schema, url.href);
  return {
    schema,
    at: (pointer = "") => ajv.getSchema(`${url.href}#${pointer}`),
  };
}
