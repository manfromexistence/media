use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

/// Icon metadata stored in zero-copy format
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct IconMetadata {
    pub id: u32,
    pub name: String,
    pub pack: String,
    pub category: String,
    pub tags: Vec<String>,
    pub popularity: u32,
}

/// Icon pack information
#[derive(SerdeDeserialize, SerdeSerialize, Debug, Clone)]
pub struct IconPack {
    pub prefix: String,
    pub info: PackInfo,
    pub icons: std::collections::HashMap<String, IconData>,
}

#[derive(SerdeDeserialize, SerdeSerialize, Debug, Clone)]
pub struct PackInfo {
    pub name: String,
    pub total: u32,
    pub author: Author,
    pub license: License,
}

#[derive(SerdeDeserialize, SerdeSerialize, Debug, Clone)]
pub struct Author {
    pub name: String,
    pub url: Option<String>,
}

#[derive(SerdeDeserialize, SerdeSerialize, Debug, Clone)]
pub struct License {
    pub title: String,
    pub spdx: Option<String>,
    pub url: Option<String>,
}

#[derive(SerdeDeserialize, SerdeSerialize, Debug, Clone)]
pub struct IconData {
    pub body: String,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
}
