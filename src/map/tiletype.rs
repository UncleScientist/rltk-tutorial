use serde::{Deserialize, Serialize};

#[derive(Eq, Hash, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}
