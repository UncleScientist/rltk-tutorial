use super::{
    AreaEndingPosition, AreaStartingPosition, BspInteriorBuilder, BuilderChain, CullUnreachable,
    VoronoiSpawning, XEnd, XStart, YEnd, YStart,
};

pub fn dark_elf_city(new_depth: i32, width: i32, height: i32) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elven City");

    chain.start_with(BspInteriorBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.with(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.with(VoronoiSpawning::new());

    chain
}
