use super::*;

use crate::map_builders::prefab_builders::prefab_sections::UNDERGROUND_FORT;

pub fn mushroom_entrance(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Mushroom Grove");

    chain.start_with(CellularAutomataBuilder::new());
    chain.with(WaveformCollapseBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.with(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(PrefabBuilder::sectional(UNDERGROUND_FORT));
    chain
}
