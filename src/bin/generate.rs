use agent_client_protocol_schema::{
    AGENT_METHOD_NAMES, AgentSide, CLIENT_METHOD_NAMES, ClientSide, JsonRpcMessage,
    OutgoingMessage, ProtocolVersion,
};
#[cfg(feature = "unstable_cancel_request")]
use agent_client_protocol_schema::{PROTOCOL_LEVEL_METHOD_NAMES, ProtocolLevelNotification};
use schemars::{
    JsonSchema,
    generate::SchemaSettings,
    transform::{RemoveRefSiblings, ReplaceBoolSchemas},
};
use std::{env, fs, path::Path};

use markdown_generator::MarkdownGenerator;

#[expect(dead_code)]
#[derive(JsonSchema)]
#[schemars(inline)]
struct AgentOutgoingMessage(JsonRpcMessage<OutgoingMessage<AgentSide, ClientSide>>);

#[expect(dead_code)]
#[derive(JsonSchema)]
#[schemars(inline)]
struct ClientOutgoingMessage(JsonRpcMessage<OutgoingMessage<ClientSide, AgentSide>>);

#[expect(dead_code)]
#[derive(JsonSchema)]
#[serde(untagged)]
#[schemars(title = "Agent Client Protocol")]
#[allow(clippy::large_enum_variant)]
enum AcpTypes {
    Agent(AgentOutgoingMessage),
    Client(ClientOutgoingMessage),
    #[cfg(feature = "unstable_cancel_request")]
    ProtocolLevel(ProtocolLevelNotification),
}

fn main() {
    let mut settings = SchemaSettings::draft2020_12();
    settings.untagged_enum_variant_titles = true;
    let mut bool_schemas = ReplaceBoolSchemas::default();
    bool_schemas.skip_additional_properties = true;
    settings = settings
        .with_transform(RemoveRefSiblings::default())
        .with_transform(bool_schemas);

    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<AcpTypes>();

    // Convert to serde_json::Value for post-processing
    let schema_value = serde_json::to_value(&schema).unwrap();

    let root = env!("CARGO_MANIFEST_DIR");
    let schema_dir = Path::new(root).join("schema");
    let docs_protocol_dir = Path::new(root).join("docs").join("protocol");

    fs::create_dir_all(schema_dir.clone()).unwrap();
    fs::create_dir_all(docs_protocol_dir.clone()).unwrap();

    let schema_file = if cfg!(feature = "unstable") {
        "schema.unstable.json"
    } else {
        "schema.json"
    };
    fs::write(
        schema_dir.join(schema_file),
        serde_json::to_string_pretty(&schema_value).unwrap(),
    )
    .unwrap_or_else(|e| panic!("Failed to write {schema_file}: {e}"));

    // Create a combined metadata object
    #[cfg(not(feature = "unstable_cancel_request"))]
    let metadata = serde_json::json!({
        "version": ProtocolVersion::LATEST,
        "agentMethods": AGENT_METHOD_NAMES,
        "clientMethods": CLIENT_METHOD_NAMES,
    });
    #[cfg(feature = "unstable_cancel_request")]
    let metadata = serde_json::json!({
        "version": ProtocolVersion::LATEST,
        "agentMethods": AGENT_METHOD_NAMES,
        "clientMethods": CLIENT_METHOD_NAMES,
        "protocolMethods": PROTOCOL_LEVEL_METHOD_NAMES,
    });

    let meta_file = if cfg!(feature = "unstable") {
        "meta.unstable.json"
    } else {
        "meta.json"
    };
    fs::write(
        schema_dir.join(meta_file),
        serde_json::to_string_pretty(&metadata).unwrap(),
    )
    .unwrap_or_else(|e| panic!("Failed to write {meta_file}: {e}"));

    // Generate markdown documentation
    let mut markdown_gen = MarkdownGenerator::new();
    let markdown_doc = markdown_gen.generate(&schema_value);

    let doc_file = if cfg!(feature = "unstable") {
        "draft/schema.mdx"
    } else {
        "schema.mdx"
    };

    fs::write(docs_protocol_dir.join(doc_file), markdown_doc)
        .unwrap_or_else(|e| panic!("Failed to write {doc_file}: {e}"));

    println!("✓ Generated {schema_file}");
    println!("✓ Generated {meta_file}");
    println!("✓ Generated {doc_file}");
}

mod markdown_generator {
    use serde_json::Value;
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Write;
    use std::fs;
    use std::process::Command;

    pub struct MarkdownGenerator {
        definitions: BTreeMap<String, Value>,
        output: String,
    }

    impl MarkdownGenerator {
        pub fn new() -> Self {
            Self {
                definitions: BTreeMap::new(),
                output: String::new(),
            }
        }

        pub fn generate(&mut self, schema: &Value) -> String {
            // Extract definitions
            if let Some(defs) = schema.get("$defs").and_then(|v| v.as_object()) {
                self.definitions = defs.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            }

            // Start with title and frontmatter
            writeln!(&mut self.output, "---").unwrap();
            writeln!(&mut self.output, "title: \"Schema\"").unwrap();
            writeln!(
                &mut self.output,
                r#"description: "Schema definitions for the Agent Client Protocol""#
            )
            .unwrap();
            writeln!(&mut self.output, "---").unwrap();
            writeln!(&mut self.output).unwrap();

            let mut agent_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            let mut client_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            let mut protocol_types: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            let mut referenced_types: Vec<(String, Value)> = Vec::new();

            for (name, def) in &self.definitions {
                if def
                    .get("x-docs-ignore")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                {
                    continue;
                }

                if let Some(side) = def.get("x-side").and_then(|v| v.as_str()) {
                    let method = def.get("x-method").unwrap().as_str().unwrap();

                    let types = match side {
                        "agent" => &mut agent_types,
                        "client" => &mut client_types,
                        "protocol" => &mut protocol_types,
                        _ => unimplemented!("Unexpected side {side}"),
                    };

                    types
                        .entry(method.to_string())
                        .or_default()
                        .push((name.clone(), def.clone()));
                } else {
                    referenced_types.push((name.clone(), def.clone()));
                }
            }

            let side_docs = extract_side_docs();

            writeln!(&mut self.output, "## Agent").unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(
                &mut self.output,
                "Defines the interface that all ACP-compliant agents must implement.

Agents are programs that use generative AI to autonomously modify code. They handle
requests from clients and execute tasks using language models and tools."
            )
            .unwrap();
            writeln!(&mut self.output).unwrap();

            for (method, types) in agent_types {
                self.generate_method(&method, side_docs.agent_method_doc(&method), types);
            }

            writeln!(&mut self.output, "## Client").unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(
                &mut self.output,
                "Defines the interface that ACP-compliant clients must implement.

Clients are typically code editors (IDEs, text editors) that provide the interface
between users and AI agents. They manage the environment, handle user interactions,
and control access to resources."
            )
            .unwrap();

            for (method, types) in client_types {
                self.generate_method(&method, side_docs.client_method_doc(&method), types);
            }
            #[cfg(feature = "unstable_cancel_request")]
            {
                writeln!(&mut self.output, "## Protocol Level").unwrap();
                writeln!(&mut self.output).unwrap();
                writeln!(
            &mut self.output,
            "Defines the interface that ACP-compliant agents and clients must both implement.

Notifications whose methods start with '$/' are messages which are protocol
implementation dependent and might not be implementable in all clients or
agents. For example if the implementation uses a single threaded synchronous
programming language then there is little it can do to react to a
`$/cancel_request` notification. If an agent or client receives notifications
starting with '$/' it is free to ignore the notification."
        )
                .unwrap();

                for (method, types) in protocol_types {
                    self.generate_method(&method, side_docs.protocol_method_doc(&method), types);
                }
            }

            referenced_types.sort_by_key(|(name, _)| name.clone());
            for (name, def) in referenced_types {
                self.document_type(2, &name, &def);
            }

            self.output.clone()
        }

        fn generate_method(
            &mut self,
            method: &str,
            docs: &str,
            mut method_types: Vec<(String, Value)>,
        ) {
            if method.contains('/') {
                writeln!(
                    &mut self.output,
                    "<a id=\"{}\"></a>",
                    Self::anchor_text(method).replace('/', "-")
                )
                .unwrap();
            }
            writeln!(
                &mut self.output,
                "### <span class=\"font-mono\">{method}</span>",
            )
            .unwrap();
            writeln!(&mut self.output).unwrap();
            writeln!(&mut self.output, "{docs}").unwrap();
            writeln!(&mut self.output).unwrap();

            method_types.sort_by_key(|(name, _)| name.clone());

            for (name, def) in method_types {
                self.document_type(4, &name, &def);
            }
        }

        fn document_type(&mut self, headline_level: usize, name: &str, definition: &Value) {
            writeln!(
                &mut self.output,
                "{} <span class=\"font-mono\">{}</span>",
                "#".repeat(headline_level),
                name,
            )
            .unwrap();
            writeln!(&mut self.output).unwrap();

            // Add main description if available
            if let Some(desc) = Self::get_def_description(definition) {
                // Escape # at the beginning of lines to prevent them from being treated as headers
                let escaped_desc = Self::escape_description(&desc);
                writeln!(&mut self.output, "{escaped_desc}").unwrap();
                writeln!(&mut self.output).unwrap();
            }
            // Determine type kind and document accordingly
            if let Some(variants) = definition
                .get("oneOf")
                .or_else(|| definition.get("anyOf"))
                .and_then(|v| v.as_array())
            {
                if variants.len() == 1 {
                    // Single-variant union: resolve the $ref and render as its
                    // underlying type instead of a "Union" wrapper.
                    let variant = &variants[0];
                    if let Some(merged_def) = self.merge_variant_definition(variant) {
                        // Preserve variant-level description if present
                        if let Some(desc) = Self::get_def_description(variant) {
                            let escaped_desc = Self::escape_description(&desc);
                            writeln!(&mut self.output, "{escaped_desc}").unwrap();
                            writeln!(&mut self.output).unwrap();
                        }
                        if merged_def.get("properties").is_some() {
                            self.document_object(&merged_def);
                        } else if let Some(type_val) =
                            merged_def.get("type").and_then(|v| v.as_str())
                        {
                            self.document_simple_type(type_val, &merged_def);
                        } else {
                            self.document_union(definition);
                        }
                    } else {
                        self.document_union(definition);
                    }
                } else {
                    self.document_union(definition);
                }
            } else if definition.get("enum").is_some() {
                self.document_enum_simple(definition);
            } else if definition.get("properties").is_some() {
                self.document_object(definition);
            } else if let Some(type_val) = definition.get("type").and_then(|v| v.as_str()) {
                self.document_simple_type(type_val, definition);
            }

            writeln!(&mut self.output).unwrap();
        }

        fn document_union(&mut self, definition: &Value) {
            writeln!(&mut self.output, "**Type:** Union").unwrap();
            writeln!(&mut self.output).unwrap();

            let discriminator_prop = definition
                .get("discriminator")
                .and_then(|d| d.get("propertyName"))
                .and_then(|p| p.as_str());

            let any_of = definition.get("anyOf").and_then(|v| v.as_array());
            let one_of = definition.get("oneOf").and_then(|v| v.as_array());

            // Union types with top-level "properties" alongside "oneOf"/"anyOf" use them
            // as shared properties that apply to all variants (e.g., _meta, message).
            // The discriminator property (if any) is excluded since it's per-variant.
            let has_shared_props = if let Some(shared_props) =
                definition.get("properties").and_then(|v| v.as_object())
            {
                let filtered_props: serde_json::Map<String, Value> = shared_props
                    .iter()
                    .filter(|(key, _)| Some(key.as_str()) != discriminator_prop)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if filtered_props.is_empty() {
                    false
                } else {
                    writeln!(&mut self.output, "**Shared properties:**").unwrap();
                    writeln!(&mut self.output).unwrap();
                    self.document_properties_as_fields(&filtered_props, definition, 0);
                    writeln!(&mut self.output).unwrap();
                    true
                }
            } else {
                false
            };

            // When both anyOf and oneOf exist (e.g. a flattened untagged enum
            // alongside a flattened tagged enum), render the anyOf variants
            // first since they represent additional constraints (like scope)
            // before the main discriminated variants.
            if let Some(variants) = any_of {
                // Label when there are other sections around this one.
                if has_shared_props || one_of.is_some() {
                    let label = if one_of.is_some() {
                        "**One of:**"
                    } else {
                        "**Variants:**"
                    };
                    writeln!(&mut self.output, "{label}").unwrap();
                    writeln!(&mut self.output).unwrap();
                }
                for variant in variants {
                    self.document_variant_table_row(variant);
                }
                writeln!(&mut self.output).unwrap();
            }

            if let Some(variants) = one_of {
                // Label when there are other sections above.
                if has_shared_props || any_of.is_some() {
                    writeln!(&mut self.output, "**Variants:**").unwrap();
                    writeln!(&mut self.output).unwrap();
                }
                for variant in variants {
                    self.document_variant_table_row(variant);
                }
                writeln!(&mut self.output).unwrap();
            }
        }

        #[expect(clippy::too_many_lines)]
        fn document_variant_table_row(&mut self, variant: &Value) {
            write!(&mut self.output, "<ResponseField name=\"").unwrap();

            // Get variant name
            if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                write!(&mut self.output, "{type_name}").unwrap();
            } else if let Some(const_val) = variant.get("const") {
                if let Some(s) = const_val.as_str() {
                    write!(&mut self.output, "{s}").unwrap();
                } else {
                    write!(&mut self.output, "{const_val}").unwrap();
                }
            } else if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
                write!(&mut self.output, "null").unwrap();
            } else if let Some(props) = variant.get("properties").and_then(|v| v.as_object()) {
                // Look for discriminator
                let discriminator = props
                    .iter()
                    .find(|(_, v)| v.get("const").is_some())
                    .and_then(|(_, v)| v.get("const").and_then(|c| c.as_str()));

                if let Some(const_val) = discriminator {
                    write!(&mut self.output, "{const_val}").unwrap();
                } else if let Some(title) = variant.get("title").and_then(|t| t.as_str()) {
                    write!(&mut self.output, "{title}").unwrap();
                } else {
                    write!(&mut self.output, "Object").unwrap();
                }
            } else if let Some(title) = variant.get("title") {
                if let Some(s) = title.as_str() {
                    write!(&mut self.output, "{s}").unwrap();
                } else {
                    write!(&mut self.output, "{title}").unwrap();
                }
            } else if let Some(ty) = variant.get("type") {
                if let Some(s) = ty.as_str() {
                    write!(&mut self.output, "{s}").unwrap();
                } else {
                    write!(&mut self.output, "{ty}").unwrap();
                }
            } else {
                write!(&mut self.output, "Variant").unwrap();
            }

            if let Some(format) = variant.get("format") {
                if let Some(s) = format.as_str() {
                    write!(&mut self.output, "\" type=\"{s}").unwrap();
                } else {
                    write!(&mut self.output, "\" type=\"{format}").unwrap();
                }
            } else if let Some(ty) = variant.get("type") {
                if let Some(s) = ty.as_str() {
                    write!(&mut self.output, "\" type=\"{s}").unwrap();
                } else {
                    write!(&mut self.output, "\" type=\"{ty}").unwrap();
                }
            }

            writeln!(&mut self.output, "\">").unwrap();

            // Get description
            if let Some(desc) = Self::get_def_description(variant) {
                writeln!(&mut self.output, "{desc}").unwrap();
            } else {
                writeln!(&mut self.output, "{{\"\"}}").unwrap();
            }

            // Collect all properties and required fields
            let mut merged_props = serde_json::Map::new();
            let mut merged_required = Vec::new();

            // Helper to merge from a definition
            let mut merge_from = |def: &Value| {
                if let Some(props) = def.get("properties").and_then(|v| v.as_object()) {
                    for (k, v) in props {
                        merged_props.insert(k.clone(), v.clone());
                    }
                }
                if let Some(req) = def.get("required").and_then(|v| v.as_array()) {
                    for r in req {
                        if !merged_required.contains(r) {
                            merged_required.push(r.clone());
                        }
                    }
                }
            };

            // 1. Check for $ref (direct)
            if let Some(merged_variant) = self.merge_variant_definition(variant) {
                merge_from(&merged_variant);
            } else {
                // 1. Check for $ref (direct)
                if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                    let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                    if let Some(ref_def) = self.definitions.get(type_name) {
                        merge_from(ref_def);
                    }
                }

                // 2. Check for allOf (often used for inheritance/composition)
                if let Some(all_of) = variant.get("allOf").and_then(|v| v.as_array()) {
                    for item in all_of {
                        if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                            let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                            if let Some(ref_def) = self.definitions.get(type_name) {
                                merge_from(ref_def);
                            }
                        } else {
                            merge_from(item);
                        }
                    }
                }

                // 3. Local properties
                merge_from(variant);
            }

            if !merged_props.is_empty() {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "<Expandable title=\"Properties\">").unwrap();
                writeln!(&mut self.output).unwrap();

                let mut synthetic_def = serde_json::Map::new();
                synthetic_def.insert("required".to_string(), Value::Array(merged_required));

                self.document_properties_as_fields(&merged_props, &Value::Object(synthetic_def), 0);
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "</Expandable>").unwrap();
            }

            writeln!(&mut self.output, "</ResponseField>").unwrap();
            writeln!(&mut self.output).unwrap();
        }

        fn document_enum_simple(&mut self, definition: &Value) {
            if let Some(enum_vals) = definition.get("enum").and_then(|v| v.as_array()) {
                writeln!(&mut self.output, "**Type:** Enumeration").unwrap();
                writeln!(&mut self.output).unwrap();

                writeln!(&mut self.output, "| Value |").unwrap();
                writeln!(&mut self.output, "| ----- |").unwrap();

                for val in enum_vals {
                    write!(&mut self.output, "| ").unwrap();
                    if let Some(s) = val.as_str() {
                        write!(&mut self.output, "`\"{s}\"`").unwrap();
                    } else {
                        write!(&mut self.output, "`{val}`").unwrap();
                    }
                    writeln!(&mut self.output, " |").unwrap();
                }
                writeln!(&mut self.output).unwrap();
            }
        }

        fn document_object(&mut self, definition: &Value) {
            writeln!(&mut self.output, "**Type:** Object").unwrap();

            if let Some(props) = definition.get("properties").and_then(|v| v.as_object()) {
                if props.is_empty() {
                    writeln!(&mut self.output).unwrap();
                    writeln!(&mut self.output, "*No properties defined*").unwrap();
                    return;
                }

                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "**Properties:**").unwrap();
                writeln!(&mut self.output).unwrap();
                self.document_properties_as_fields(props, definition, 0);
            }
        }

        fn document_properties_as_fields(
            &mut self,
            props: &serde_json::Map<String, Value>,
            definition: &Value,
            indent: usize,
        ) {
            let indent_str = " ".repeat(indent);

            // Get required fields
            let required = definition
                .get("required")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();

            // Sort properties for consistent output
            let mut sorted_props: Vec<(&String, &Value)> = props.iter().collect();
            sorted_props.sort_by_key(|(name, _)| name.as_str());

            for (prop_name, prop_schema) in sorted_props {
                let is_required = required.contains(&prop_name.as_str());
                let type_str = Self::get_type_string(prop_schema);

                // Simple field without nesting
                writeln!(
                    &mut self.output,
                    "{}<ResponseField name=\"{}\" type={{{}}} {}>",
                    indent_str,
                    prop_name,
                    type_str,
                    if is_required { "required" } else { "" }
                )
                .unwrap();

                // Add description if available
                if let Some(desc) = Self::get_def_description(prop_schema) {
                    writeln!(&mut self.output, "{indent_str}  {desc}").unwrap();
                } else if let Some(const_val) = prop_schema.get("const") {
                    let val_str = if let Some(s) = const_val.as_str() {
                        format!("\"{s}\"")
                    } else {
                        const_val.to_string()
                    };
                    writeln!(
                        &mut self.output,
                        "{indent_str}  The discriminator value. Must be `{val_str}`."
                    )
                    .unwrap();
                }

                // Add constraints if any
                self.document_field_constraints(prop_schema, indent + 2);

                writeln!(&mut self.output, "{indent_str}</ResponseField>").unwrap();
            }
        }

        fn document_field_constraints(&mut self, schema: &Value, indent: usize) {
            let indent_str = " ".repeat(indent);
            let mut constraints = Vec::new();

            if let Some(v) = schema.get("default") {
                constraints.push((
                    "Default",
                    format!("`{}`", serde_json::to_string(v).unwrap_or_default()),
                ));
            }
            if let Some(v) = schema.get("minimum") {
                constraints.push(("Minimum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maximum") {
                constraints.push(("Maximum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("minLength") {
                constraints.push(("Min length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maxLength") {
                constraints.push(("Max length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("pattern") {
                constraints.push(("Pattern", format!("`{v}`")));
            }

            if !constraints.is_empty() {
                writeln!(&mut self.output).unwrap();
                if constraints.len() == 1 {
                    // Single constraint as text
                    let (name, value) = &constraints[0];
                    writeln!(&mut self.output, "{indent_str}  - {name}: {value}").unwrap();
                } else {
                    // Multiple constraints as table
                    writeln!(&mut self.output, "{indent_str}  | Constraint | Value |").unwrap();
                    writeln!(&mut self.output, "{indent_str}  | ---------- | ----- |").unwrap();
                    for (name, value) in constraints {
                        writeln!(&mut self.output, "{indent_str}  | {name} | {value} |").unwrap();
                    }
                }
            }

            // Document enum values if present
            if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "{indent_str}  **Allowed values:**").unwrap();
                for val in enum_vals {
                    if let Some(s) = val.as_str() {
                        writeln!(&mut self.output, "{indent_str}  - `\"{s}\"`").unwrap();
                    } else {
                        writeln!(&mut self.output, "{indent_str}  - `{val}`").unwrap();
                    }
                }
            }
        }

        fn document_simple_type(&mut self, type_name: &str, definition: &Value) {
            let formatted_type = match type_name {
                "integer" => {
                    if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                        format!("integer ({format})")
                    } else {
                        "integer".to_string()
                    }
                }
                "number" => {
                    if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                        format!("number ({format})")
                    } else {
                        "number".to_string()
                    }
                }
                "string" => {
                    if let Some(format) = definition.get("format").and_then(|v| v.as_str()) {
                        format!("string ({format})")
                    } else {
                        "string".to_string()
                    }
                }
                _ => type_name.to_string(),
            };

            writeln!(&mut self.output, "**Type:** `{formatted_type}`").unwrap();

            // Document constraints if any
            self.document_constraints(definition);
        }

        fn document_constraints(&mut self, schema: &Value) {
            let mut constraints = Vec::new();

            if let Some(v) = schema.get("default") {
                constraints.push((
                    "Default",
                    format!("`{}`", serde_json::to_string(v).unwrap_or_default()),
                ));
            }
            if let Some(v) = schema.get("minimum") {
                constraints.push(("Minimum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maximum") {
                constraints.push(("Maximum", format!("`{v}`")));
            }
            if let Some(v) = schema.get("minLength") {
                constraints.push(("Min length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("maxLength") {
                constraints.push(("Max length", format!("`{v}`")));
            }
            if let Some(v) = schema.get("pattern") {
                constraints.push(("Pattern", format!("`{v}`")));
            }
            if let Some(v) = schema.get("format").and_then(|v| v.as_str())
                && !["int32", "int64", "uint16", "uint32", "uint64", "double"].contains(&v)
            {
                constraints.push(("Format", format!("`{v}`")));
            }

            if !constraints.is_empty() {
                writeln!(&mut self.output).unwrap();
                if constraints.len() == 1 {
                    // Single constraint as text
                    let (name, value) = &constraints[0];
                    writeln!(&mut self.output, "**{name}:** {value}").unwrap();
                } else {
                    // Multiple constraints as table
                    writeln!(&mut self.output, "| Constraint | Value |").unwrap();
                    writeln!(&mut self.output, "| ---------- | ----- |").unwrap();
                    for (name, value) in constraints {
                        writeln!(&mut self.output, "| {name} | {value} |").unwrap();
                    }
                }
            }

            // Document enum values if present
            if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
                writeln!(&mut self.output).unwrap();
                writeln!(&mut self.output, "| Allowed Values |").unwrap();
                writeln!(&mut self.output, "| -------------- |").unwrap();
                for val in enum_vals {
                    write!(&mut self.output, "| ").unwrap();
                    if let Some(s) = val.as_str() {
                        write!(&mut self.output, "`\"{s}\"`").unwrap();
                    } else {
                        write!(&mut self.output, "`{val}`").unwrap();
                    }
                    writeln!(&mut self.output, " |").unwrap();
                }
            }
        }

        fn get_ref_type_name(schema: &Value) -> Option<&str> {
            if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
                return Some(ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val));
            }

            // Check for single-item allOf/anyOf/oneOf wrappers (often used for $ref with sibling properties)
            for key in ["allOf", "anyOf", "oneOf"] {
                if let Some(arr) = schema.get(key).and_then(|v| v.as_array())
                    && arr.len() == 1
                    && let Some(type_name) = Self::get_ref_type_name(&arr[0])
                {
                    return Some(type_name);
                }
            }

            None
        }

        fn get_array_type_string(schema: &Value) -> String {
            if let Some(items) = schema.get("items") {
                if let Some(type_name) = Self::get_ref_type_name(items) {
                    return format!(
                        "<a href=\"#{}\">{}[]</a>",
                        MarkdownGenerator::anchor_text(type_name),
                        type_name
                    );
                }

                let item_type = MarkdownGenerator::get_type_string(items);
                format!("<><span>{item_type}</span><span>[]</span></>")
            } else {
                "\"array\"".to_string()
            }
        }

        fn get_type_string(schema: &Value) -> String {
            // Check for $ref
            if let Some(ref_val) = schema.get("$ref").and_then(|v| v.as_str()) {
                let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                return format!(
                    "<a href=\"#{}\">{}</a>",
                    Self::anchor_text(type_name),
                    type_name
                );
            }

            // Check for single-item allOf/anyOf/oneOf wrappers (often used for $ref with sibling properties)
            for key in ["allOf", "anyOf", "oneOf"] {
                if let Some(arr) = schema.get(key).and_then(|v| v.as_array())
                    && arr.len() == 1
                {
                    return Self::get_type_string(&arr[0]);
                }
            }

            // Check for type
            if let Some(type_val) = schema.get("type") {
                if let Some(type_str) = type_val.as_str() {
                    return match type_str {
                        "array" => Self::get_array_type_string(schema),
                        "integer" => {
                            let type_str = if let Some(format) =
                                schema.get("format").and_then(|v| v.as_str())
                            {
                                format
                            } else {
                                type_str
                            };
                            format!("\"{type_str}\"")
                        }
                        _ => format!("\"{type_str}\""),
                    };
                }

                // Handle multiple types (nullable)
                if let Some(arr) = type_val.as_array() {
                    let types: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
                    if types.is_empty() {
                        return "\"object\"".to_string();
                    }

                    // Special-case nullable arrays so we can still show the item type (and link to it).
                    if types.contains(&"array") && schema.get("items").is_some() {
                        let array_type = Self::get_array_type_string(schema);
                        let rest: Vec<&str> =
                            types.iter().copied().filter(|t| *t != "array").collect();
                        if rest.is_empty() {
                            return array_type;
                        }
                        let rest_text = rest.join(" | ");
                        return format!(
                            "<><span>{array_type}</span><span> | {rest_text}</span></>"
                        );
                    }

                    return format!("\"{}\"", types.join(" | "));
                }
            }

            // Check for oneOf/anyOf
            if schema.get("oneOf").is_some() || schema.get("anyOf").is_some() {
                // Try to get more specific union type info
                if let Some(variants) = schema.get("oneOf").or_else(|| schema.get("anyOf"))
                    && let Some(arr) = variants.as_array()
                    && arr.len() == 2
                {
                    // Check for nullable pattern (type | null)
                    let mut has_null = false;
                    let mut other_type = None;
                    for variant in arr {
                        if variant.get("type").and_then(|v| v.as_str()) == Some("null") {
                            has_null = true;
                        } else if let Some(t) = Self::get_inline_variant_type(variant) {
                            other_type = Some(t);
                        }
                    }
                    if has_null && let Some(other_type) = other_type {
                        return format!("<><span>{other_type}</span><span> | null</span></>");
                    }
                }
                return "union".to_string();
            }

            // Check for enum
            if schema.get("enum").is_some() {
                return "\"enum\"".to_string();
            }

            "\"object\"".to_string()
        }

        fn get_inline_variant_type(variant: &Value) -> Option<String> {
            if variant.get("oneOf").is_some() || variant.get("anyOf").is_some() {
                return None;
            }

            // Check for simple type
            if variant.get("type").and_then(|v| v.as_str()).is_some() {
                return Some(Self::get_type_string(variant));
            }
            // Check for $ref
            if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                return Some(format!(
                    "<a href=\"#{}\">{}</a>",
                    Self::anchor_text(type_name),
                    type_name
                ));
            }
            None
        }

        fn escape_mdx(text: &str) -> String {
            text.replace('|', "\\|")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('{', "\\{")
                .replace('}', "\\}")
        }

        fn escape_description(text: &str) -> String {
            // Escape # at the beginning of lines to prevent them from being treated as headers
            let lines: Vec<String> = text
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with('#') {
                        // Escape the # by replacing it with \#
                        let trimmed_start = line.len() - line.trim_start().len();
                        format!("{}\\{}", &line[..trimmed_start], &line[trimmed_start..])
                    } else {
                        line.to_string()
                    }
                })
                .collect();
            lines.join("\n")
        }

        fn get_def_description(def: &Value) -> Option<String> {
            let desc = def
                .get("description")?
                .as_str()?
                .replace("[`", "`")
                .replace("`]", "`");
            let desc = Self::escape_mdx(&desc);
            Some(desc)
        }

        fn merge_variant_definition(&self, variant: &Value) -> Option<Value> {
            let mut merged = if let Some(ref_val) = variant.get("$ref").and_then(|v| v.as_str()) {
                let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                self.definitions.get(type_name).cloned()?
            } else if let Some(all_of) = variant.get("allOf").and_then(|v| v.as_array()) {
                let mut base = None;

                for item in all_of {
                    if let Some(ref_val) = item.get("$ref").and_then(|v| v.as_str()) {
                        let type_name = ref_val.strip_prefix("#/$defs/").unwrap_or(ref_val);
                        if let Some(def) = self.definitions.get(type_name) {
                            base = Some(def.clone());
                            break;
                        }
                    }
                }

                base.unwrap_or_else(|| Value::Object(serde_json::Map::new()))
            } else {
                return None;
            };

            let Some(merged_obj) = merged.as_object_mut() else {
                return Some(merged);
            };

            let mut wrapper_props = serde_json::Map::new();
            let mut wrapper_required = Vec::new();

            let mut collect_fields = |schema: &Value| {
                if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                    for (key, value) in props {
                        wrapper_props
                            .entry(key.clone())
                            .or_insert_with(|| value.clone());
                    }
                }
                if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
                    for req in required {
                        if !wrapper_required.contains(req) {
                            wrapper_required.push(req.clone());
                        }
                    }
                }
            };

            if let Some(all_of) = variant.get("allOf").and_then(|v| v.as_array()) {
                for item in all_of {
                    if item.get("$ref").is_none() {
                        collect_fields(item);
                    }
                }
            }

            collect_fields(variant);

            if !wrapper_props.is_empty() {
                let target_props = merged_obj
                    .entry("properties".to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                if let Some(target_props_obj) = target_props.as_object_mut() {
                    for (key, value) in wrapper_props {
                        target_props_obj.entry(key).or_insert(value);
                    }
                }
            }

            if !wrapper_required.is_empty() {
                let target_required = merged_obj
                    .entry("required".to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Some(target_required_arr) = target_required.as_array_mut() {
                    for req in wrapper_required {
                        if !target_required_arr.contains(&req) {
                            target_required_arr.push(req);
                        }
                    }
                }
            }

            Some(merged)
        }

        fn anchor_text(title: &str) -> String {
            title.to_lowercase()
        }
    }

    #[derive(Default)]
    struct SideDocs {
        agent: HashMap<String, String>,
        client: HashMap<String, String>,
        protocol: HashMap<String, String>,
    }

    impl SideDocs {
        fn agent_method_doc(&self, method_name: &str) -> &String {
            match method_name {
                "initialize" => self.agent.get("InitializeRequest").unwrap(),
                "authenticate" => self.agent.get("AuthenticateRequest").unwrap(),
                "session/new" => self.agent.get("NewSessionRequest").unwrap(),
                "session/load" => self.agent.get("LoadSessionRequest").unwrap(),
                "session/list" => self.agent.get("ListSessionsRequest").unwrap(),
                "session/fork" => self.agent.get("ForkSessionRequest").unwrap(),
                "session/resume" => self.agent.get("ResumeSessionRequest").unwrap(),
                "session/set_mode" => self.agent.get("SetSessionModeRequest").unwrap(),
                "session/set_config_option" => {
                    self.agent.get("SetSessionConfigOptionRequest").unwrap()
                }
                "session/prompt" => self.agent.get("PromptRequest").unwrap(),
                "session/cancel" => self.agent.get("CancelNotification").unwrap(),
                "session/set_model" => self.agent.get("SetSessionModelRequest").unwrap(),
                "session/close" => self.agent.get("CloseSessionRequest").unwrap(),
                "logout" => self.agent.get("LogoutRequest").unwrap(),
                "nes/start" => self.agent.get("StartNesRequest").unwrap(),
                "nes/suggest" => self.agent.get("SuggestNesRequest").unwrap(),
                "nes/close" => self.agent.get("CloseNesRequest").unwrap(),
                "nes/accept" => self.agent.get("AcceptNesNotification").unwrap(),
                "nes/reject" => self.agent.get("RejectNesNotification").unwrap(),
                "document/didOpen" => self.agent.get("DidOpenDocumentNotification").unwrap(),
                "document/didChange" => self.agent.get("DidChangeDocumentNotification").unwrap(),
                "document/didClose" => self.agent.get("DidCloseDocumentNotification").unwrap(),
                "document/didSave" => self.agent.get("DidSaveDocumentNotification").unwrap(),
                "document/didFocus" => self.agent.get("DidFocusDocumentNotification").unwrap(),
                _ => panic!("Introduced a method? Add it here :)"),
            }
        }

        fn client_method_doc(&self, method_name: &str) -> &String {
            match method_name {
                "session/request_permission" => {
                    self.client.get("RequestPermissionRequest").unwrap()
                }
                "fs/write_text_file" => self.client.get("WriteTextFileRequest").unwrap(),
                "fs/read_text_file" => self.client.get("ReadTextFileRequest").unwrap(),
                "session/update" => self.client.get("SessionNotification").unwrap(),
                "terminal/create" => self.client.get("CreateTerminalRequest").unwrap(),
                "terminal/output" => self.client.get("TerminalOutputRequest").unwrap(),
                "terminal/release" => self.client.get("ReleaseTerminalRequest").unwrap(),
                "terminal/wait_for_exit" => self.client.get("WaitForTerminalExitRequest").unwrap(),
                "terminal/kill" => self.client.get("KillTerminalRequest").unwrap(),
                #[cfg(feature = "unstable_elicitation")]
                "elicitation/create" => self.client.get("CreateElicitationRequest").unwrap(),
                #[cfg(feature = "unstable_elicitation")]
                "elicitation/complete" => {
                    self.client.get("CompleteElicitationNotification").unwrap()
                }
                _ => panic!("Introduced a method? Add it here :)"),
            }
        }

        #[cfg(feature = "unstable_cancel_request")]
        fn protocol_method_doc(&self, method_name: &str) -> &String {
            match method_name {
                "$/cancel_request" => self.protocol.get("CancelRequestNotification").unwrap(),
                _ => panic!("Introduced a method? Add it here :)"),
            }
        }
    }

    fn extract_side_docs() -> SideDocs {
        let output = Command::new("cargo")
            .args([
                "+nightly",
                "rustdoc",
                "--lib",
                "--all-features",
                "--",
                "-Z",
                "unstable-options",
                "--output-format",
                "json",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "Failed to generate rustdoc JSON: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Parse the JSON output
        let json_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/doc/agent_client_protocol_schema.json");
        let json_content = fs::read_to_string(json_path).unwrap();
        let doc: Value = serde_json::from_str(&json_content).unwrap();

        let mut side_docs = SideDocs::default();

        if let Some(index) = doc["index"].as_object() {
            for (_, item) in index {
                if item["name"].as_str() == Some("ClientRequest")
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.agent.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("ClientNotification")
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.agent.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("AgentRequest")
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.client.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("AgentNotification")
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.client.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }

                if item["name"].as_str() == Some("ProtocolLevelNotification")
                    && let Some(variants) = item["inner"]["enum"]["variants"].as_array()
                {
                    for variant_id in variants {
                        if let Some(variant) = doc["index"][variant_id.to_string()].as_object()
                            && let Some(name) = variant["name"].as_str()
                        {
                            side_docs.protocol.insert(
                                name.to_string(),
                                variant["docs"].as_str().unwrap_or_default().to_string(),
                            );
                        }
                    }
                }
            }
        }

        side_docs
    }

    #[cfg(test)]
    mod tests {
        use super::MarkdownGenerator;
        use serde_json::json;

        #[test]
        fn document_union_includes_shared_properties() {
            let mut generator = MarkdownGenerator::new();
            let definition = json!({
                "description": "Example union.",
                "discriminator": {
                    "propertyName": "mode"
                },
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Shared message."
                    },
                    "mode": {
                        "type": "string",
                        "description": "The discriminator."
                    }
                },
                "required": ["message", "mode"],
                "oneOf": [
                    {
                        "description": "First variant.",
                        "properties": {
                            "mode": {
                                "const": "form",
                                "type": "string"
                            }
                        },
                        "required": ["mode"],
                        "type": "object"
                    },
                    {
                        "description": "Second variant.",
                        "properties": {
                            "mode": {
                                "const": "url",
                                "type": "string"
                            }
                        },
                        "required": ["mode"],
                        "type": "object"
                    }
                ]
            });

            generator.document_type(4, "ExampleUnion", &definition);

            assert!(generator.output.contains("**Shared properties:**"));
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"message\" type={\"string\"} required>")
            );
            assert!(generator.output.contains("Shared message."));
            assert!(generator.output.contains("**Variants:**"));
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"form\" type=\"object\">")
            );
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"url\" type=\"object\">"),
            );
            let shared_section = generator.output.split("**Variants:**").next().unwrap_or("");
            assert!(
                !shared_section.contains("<ResponseField name=\"mode\""),
                "discriminator property 'mode' should not appear in shared properties"
            );
        }

        #[test]
        fn document_union_renders_both_any_of_and_one_of() {
            let mut generator = MarkdownGenerator::new();
            let definition = json!({
                "description": "Request with scope and mode.",
                "anyOf": [
                    {
                        "description": "Session scope.",
                        "properties": {
                            "sessionId": { "type": "string" }
                        },
                        "required": ["sessionId"],
                        "title": "Session",
                        "type": "object"
                    },
                    {
                        "description": "Request scope.",
                        "properties": {
                            "requestId": { "type": "integer" }
                        },
                        "required": ["requestId"],
                        "title": "Request",
                        "type": "object"
                    }
                ],
                "discriminator": { "propertyName": "mode" },
                "oneOf": [
                    {
                        "description": "Form variant.",
                        "properties": {
                            "mode": { "const": "form", "type": "string" }
                        },
                        "required": ["mode"],
                        "type": "object"
                    },
                    {
                        "description": "URL variant.",
                        "properties": {
                            "mode": { "const": "url", "type": "string" }
                        },
                        "required": ["mode"],
                        "type": "object"
                    }
                ],
                "properties": {
                    "message": { "type": "string", "description": "A message." }
                },
                "required": ["message"],
                "type": "object"
            });

            generator.document_type(4, "TestRequest", &definition);

            // Shared properties rendered
            assert!(generator.output.contains("**Shared properties:**"));
            assert!(generator.output.contains("<ResponseField name=\"message\""));

            // anyOf scope variants use title, not "Object"
            assert!(
                generator.output.contains("**One of:**"),
                "anyOf variants should be labeled 'One of:' when oneOf also exists"
            );
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"Session\" type=\"object\">"),
                "should use title 'Session' not 'Object'"
            );
            assert!(
                generator
                    .output
                    .contains("<ResponseField name=\"Request\" type=\"object\">"),
                "should use title 'Request' not 'Object'"
            );

            // oneOf mode variants rendered under Variants
            assert!(generator.output.contains("**Variants:**"));
            assert!(generator.output.contains("<ResponseField name=\"form\""));
            assert!(generator.output.contains("<ResponseField name=\"url\""));

            // Verify ordering: One of → Session/Request → Variants → form/url
            let one_of_pos = generator.output.find("**One of:**").unwrap();
            let session_pos = generator.output.find("\"Session\"").unwrap();
            let variants_pos = generator.output.find("**Variants:**").unwrap();
            let form_pos = generator.output.find("\"form\"").unwrap();
            assert!(one_of_pos < session_pos);
            assert!(session_pos < variants_pos);
            assert!(variants_pos < form_pos);
        }
    }
}
