use serde::{Deserialize, Serialize};

#[derive(Eq, Hash, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Stalactite,
    Stalagmite,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs,
}

use TileType::*;

pub fn tile_walkable(tt: TileType) -> bool {
    matches!(
        tt,
        Floor | DownStairs | Road | Grass | ShallowWater | WoodFloor | Bridge | Gravel | UpStairs
    )
}

pub fn tile_opaque(tt: TileType) -> bool {
    tt == Wall || tt == Stalactite || tt == Stalagmite
}

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        Road => 0.8,
        Grass => 1.1,
        ShallowWater => 1.2,
        _ => 1.0,
    }
}
