use rltk::RandomNumberGenerator;

use super::*;

pub fn dwarf_fort_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarven Fortress");

    chain.start_with(BspDungeonBuilder::new());
    chain.with(RoomSorter::new(RoomSort::Central));
    chain.with(RoomDrawer::new());
    chain.with(BspCorridors::new());
    chain.with(CorridorSpawner::new());
    chain.with(DragonsLair::new());

    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Top));
    chain.with(CullUnreachable::new());
    chain.with(AreaEndingPosition::new(XEnd::Right, YEnd::Bottom));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());

    chain
}

pub struct DragonsLair {}

impl MetaMapBuilder for DragonsLair {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DragonsLair {
    pub fn new() -> Box<DragonsLair> {
        Box::new(DragonsLair {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // build_data.map.depth = 6;
        build_data.take_snapshot();

        let mut builder = BuilderChain::new(6, build_data.width, build_data.height, "New Map");
        builder.start_with(DLABuilder::insectoid());
        builder.build_map(rng);

        // Add the history to our history
        for h in builder.build_data.history.iter() {
            build_data.history.push(h.clone());
        }
        build_data.take_snapshot();

        // Merge the maps
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            if *tt == TileType::Wall && builder.build_data.map.tiles[idx] == TileType::Floor {
                *tt = TileType::Floor;
            }
        }
    }
}
