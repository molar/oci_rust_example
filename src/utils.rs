use std::{collections::HashMap, path::Path};

pub fn read_kv_file(path: &Path) -> std::io::Result<HashMap<String, String>> {
    let env_string = std::fs::read_to_string(path)?;
    let kv_pairs: HashMap<String, String> = env_string
        .lines()
        .filter_map(|line| line.split_once("="))
        .map(|kv| (String::from(kv.0), String::from(kv.1)))
        .collect();
    Ok(kv_pairs)
}
