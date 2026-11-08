// Imports
use p2d::math::Vector2;
use serde::{Deserialize, Serialize};

/// A bookmark on the document canvas, marking a position to jump to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename = "bookmark")]
pub struct Bookmark {
    #[serde(rename = "name")]
    pub name: String,
    /// The marked position in document coordinate space.
    #[serde(rename = "pos")]
    pub pos: Vector2,
}

impl Default for Bookmark {
    fn default() -> Self {
        Self {
            name: String::from("Bookmark"),
            pos: Vector2::ZERO,
        }
    }
}

impl Bookmark {
    pub fn new(name: String, pos: Vector2) -> Self {
        Self { name, pos }
    }
}
