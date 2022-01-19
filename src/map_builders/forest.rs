use super::{
    AreaStartingPosition, BuilderChain, CellularAutomataBuilder, CullUnreachable, DistantExit,
    VoronoiSpawning, XStart, YStart,
};
use rltk::RandomNumberGenerator;

pub fn forest_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));

    // Set up an exit and spawn mobs
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());

    chain
}
