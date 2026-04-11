use std::path::Path;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Node {
    _date: u64,
    id: u64,
    x: f64,
    y: f64,
    z: f64,
    #[serde(deserialize_with = "uppercase_bool")]
    active: bool,
}

fn uppercase_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let string: &str = <&str>::deserialize(deserializer)?;
    match string {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Err(D::Error::custom("not a boolean")),
    }
}

pub fn read_nodes(path: impl AsRef<Path>) -> Vec<Node> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap();
    reader.deserialize().map(|result| result.unwrap()).collect()
}
