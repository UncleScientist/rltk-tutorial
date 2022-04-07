pub mod ai;
pub use ai::*;

pub mod dispatcher;
pub use dispatcher::*;

pub mod hunger_system;
pub use hunger_system::*;

pub mod inventory_system;
pub use inventory_system::*;

pub mod lighting_system;
pub use lighting_system::*;

pub mod map_indexing_system;
pub use map_indexing_system::*;

pub mod melee_combat_system;
pub use melee_combat_system::*;

pub mod movement_system;
pub use movement_system::*;

pub mod particle_system;
pub use particle_system::*;

pub mod ranged_combat_system;
pub use ranged_combat_system::*;

pub mod trigger_system;
pub use trigger_system::*;

pub mod visibility_system;
pub use visibility_system::*;

pub fn build() -> Box<dyn UnifiedDispatcher + 'static> {
    dispatcher::new()
}
