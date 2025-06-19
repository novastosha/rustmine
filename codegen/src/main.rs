use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use itertools::Itertools;
use serde_json::Value;

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<String>()
}

fn main() {
    let blocks_path = Path::new("./codegen/generator/generated/blocks.json");
    let registries_path = Path::new("./codegen/generator/generated/registries.json");
    let biomes_dir = Path::new("./codegen/generator/generated/biomes");

    let output_path = Path::new("./rustmine_lib/src/blocks.rs");

    // --- BLOCKS + PROTOCOL ID ---
    let blocks_json: Value =
        serde_json::from_str(&fs::read_to_string(blocks_path).expect("Failed to read blocks.json"))
            .expect("Invalid blocks.json");

    let registries_json: Value = serde_json::from_str(
        &fs::read_to_string(registries_path).expect("Failed to read registries.json"),
    )
    .expect("Invalid registries.json");

    let block_registry = &registries_json["minecraft:block"]["entries"];

    let mut enum_variants = String::new();
    let mut registry_entries = String::new();

    for (block_name, block_info) in blocks_json.as_object().unwrap() {
        let enum_name = pascal_case(block_name.strip_prefix("minecraft:").unwrap());

        enum_variants.push_str(&format!("    {},\n", enum_name));

        let states = block_info["states"].as_array().unwrap();
        let mut state_entries = String::new();
        let mut default_state = None;

        for state in states {
            let id = state["id"].as_u64().unwrap();
            let props = state["properties"]
                .as_object()
                .unwrap_or(&serde_json::Map::default())
                .iter()
                .map(|(k, v)| format!("(\"{}\", \"{}\")", k, v.as_str().unwrap()))
                .collect::<Vec<_>>()
                .join(", ");

            state_entries.push_str(&format!(
                "        BlockState {{ id: {}, properties: &[{}] }},\n",
                id, props
            ));

            if state.get("default").is_some() && state["default"].as_bool().unwrap() {
                default_state = Some(id);
            }
        }

        let default_id = default_state.unwrap_or(states[0]["id"].as_u64().unwrap());
        let protocol_id = block_registry[block_name]["protocol_id"].as_u64().unwrap();

        registry_entries.push_str(&format!(
            "    Block::{0} => BlockRegistryEntry {{\n        name: \"{1}\",\n        states: &[\n{2}        ],\n        default_state: {3},\n        protocol_id: {4},\n    }},\n",
            enum_name, block_name, state_entries, default_id, protocol_id
        ));
    }

    let generated = format!(
        r#"
// AUTO-GENERATED FILE. DO NOT EDIT.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {{
{enum_variants}}}

pub struct BlockState {{
    pub id: u32,
    pub properties: &'static [(&'static str, &'static str)],
}}

pub struct BlockRegistryEntry {{
    pub name: &'static str,
    pub states: &'static [BlockState],
    pub default_state: u64,
    pub protocol_id: u64,
}}

pub fn get_block_registry_entry(block: Block) -> BlockRegistryEntry {{
    match block {{
{registry_entries}    }}
}}
"#,
        enum_variants = enum_variants,
        registry_entries = registry_entries,
    );

    fs::write(output_path, generated).unwrap();
    println!("Generated: {}", output_path.display());

    // --- BIOMES + PARAMETERS ---
    let output_path = Path::new("./rustmine_lib/src/biomes.rs");
    let mut biome_enum_variants = HashSet::new();
    let mut biome_param_map: HashMap<String, Vec<String>> = HashMap::new();

    for entry in fs::read_dir(biomes_dir).expect("Failed to read biomes dir") {
        let path = entry.expect("Invalid entry").path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let contents = fs::read_to_string(&path).expect("Failed to read biome file");
            let biome_file: Value = serde_json::from_str(&contents).expect("Invalid biome file");

            if let Some(array) = biome_file["biomes"].as_array() {
                for biome_entry in array {
                    let name = biome_entry["biome"].as_str().unwrap();
                    biome_enum_variants.insert(name.to_string());

                    let enum_name = pascal_case(name.strip_prefix("minecraft:").unwrap());
                    let p = &biome_entry["parameters"];

                    let format_range = |v: &Value| -> String {
                        if let Some(arr) = v.as_array() {
                            format!("[{}, {}]", arr[0], arr[1])
                        } else {
                            "[0.0, 0.0]".to_string()
                        }
                    };

                    let parse_range_or_scalar = |v: &Value| -> String {
                        if let Some(arr) = v.as_array() {
                            format!("[{}, {}]", arr[0], arr[1])
                        } else {
                            let val = v.as_f64().unwrap_or(0.0);
                            format!("[{}, {}]", val, val)
                        }
                    };

                    let param_code = format!(
                        "        BiomeParameters {{ depth: {}, offset: {}, temperature: {}, humidity: {}, continentalness: {}, erosion: {}, weirdness: {} }},\n",
                        parse_range_or_scalar(&p["depth"]),
                        parse_range_or_scalar(&p["offset"]),
                        format_range(&p["temperature"]),
                        format_range(&p["humidity"]),
                        format_range(&p["continentalness"]),
                        format_range(&p["erosion"]),
                        format_range(&p["weirdness"]),
                    );

                    biome_param_map
                        .entry(enum_name)
                        .or_default()
                        .push(param_code);
                }
            }
        }
    }

    let mut biome_enum_str = String::new();
    let mut biome_lookup_match_arms = String::new();

    for biome in biome_enum_variants.iter().sorted() {
        let enum_name = pascal_case(biome.strip_prefix("minecraft:").unwrap());

        biome_enum_str.push_str(&format!("    {},\n", enum_name));

        let all_params = biome_param_map.get(&enum_name).unwrap_or(&vec![]).join("");
        biome_lookup_match_arms.push_str(&format!(
            "        Biome::{} => &[\n{}        ],\n",
            enum_name, all_params
        ));
    }

    let generated = format!(
        r#"
// AUTO-GENERATED FILE. DO NOT EDIT.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {{
{biome_enum_str}}}

#[derive(Debug)]
pub struct BiomeParameters {{
    pub depth: [f32; 2],
    pub offset: [f32; 2],
    pub temperature: [f32; 2],
    pub humidity: [f32; 2],
    pub continentalness: [f32; 2],
    pub erosion: [f32; 2],
    pub weirdness: [f32; 2],
}}

pub fn get_biome_parameters(biome: Biome) -> &'static [BiomeParameters] {{
    match biome {{
{biome_lookup_match_arms}    }}
}}
"#,
        biome_enum_str = biome_enum_str,
        biome_lookup_match_arms = biome_lookup_match_arms
    );

    fs::write(output_path, generated).unwrap();
    println!("Generated: {}", output_path.display());
}
