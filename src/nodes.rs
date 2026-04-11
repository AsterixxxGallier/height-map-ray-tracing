use std::path::Path;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use crate::transform::ModelSpacePosition;

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Node {
    pub _date: u64,
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[serde(deserialize_with = "uppercase_bool")]
    pub active: bool,
}

impl Node {
    pub fn position(&self) -> ModelSpacePosition {
        ModelSpacePosition {
            x: self.x,
            y: self.y,
        }
    }
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
