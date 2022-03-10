use std::collections::HashMap;

use crate::attr_bonus;
use crate::{Map, MasterDungeonMap};
use rltk::{Point, RGB};
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs_derive::{Component, ConvertSaveload};

pub struct SerializeMe;

#[derive(Serialize, Deserialize, Clone)]
pub struct SpecialAbility {
    pub spell: String,
    pub chance: f32,
    pub range: f32,
    pub min_range: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpecialAbilities {
    pub abilities: Vec<SpecialAbility>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Slow {
    pub initiative_penalty: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DamageOverTime {
    pub damage: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct TeachesSpell {
    pub spell: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesMana {
    pub mana_amount: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KnownSpell {
    pub display_name: String,
    pub mana_cost: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct KnownSpells {
    pub spells: Vec<KnownSpell>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpellTemplate {
    pub mana_cost: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct TileSize {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToCastSpell {
    pub spell: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct AttributeBonus {
    pub might: Option<i32>,
    pub fitness: Option<i32>,
    pub quickness: Option<i32>,
    pub intelligence: Option<i32>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesIdentification {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesRemoveCurse {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct CursedItem {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleLine {
    pub glyph: rltk::FontCharType,
    pub color: RGB,
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph: rltk::FontCharType,
    pub color: RGB,
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ApplyMove {
    pub dest_idx: usize,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ApplyTeleport {
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_depth: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct TownPortal {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct TeleportTo {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub player_only: bool,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Vendor {
    pub categories: Vec<String>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Chasing {
    pub target: Entity,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToApproach {
    pub idx: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct WantsToFlee {
    pub indices: Vec<usize>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Faction {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MyTurn {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Initiative {
    pub current: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct LightSource {
    pub color: RGB,
    pub range: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub table: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct NaturalAttackDefense {
    pub armor_class: Option<i32>,
    pub attacks: Vec<NaturalAttack>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EquipmentChanged {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub level: i32,
    pub total_weight: f32,
    pub total_initiative_penalty: f32,
    pub gold: f32,
    pub god_mode: bool,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum MagicItemClass {
    Common,
    Rare,
    Legendary,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct IdentifiedItem {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MagicItem {
    pub class: MagicItemClass,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ObfuscatedName {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Movement {
    Static,
    Random,
    RandomWaypoint { path: Option<Vec<usize>> },
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MoveMode {
    pub mode: Movement,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Skill {
    Melee,
    Defense,
    Magic,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub skills: HashMap<Skill, i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

impl Default for Attribute {
    fn default() -> Attribute {
        Attribute::new()
    }
}

impl Attribute {
    pub fn new() -> Attribute {
        Attribute::new_base(11)
    }

    pub fn new_base(base: i32) -> Attribute {
        Attribute {
            base,
            modifiers: 0,
            bonus: attr_bonus(base),
        }
    }
}

#[derive(Default, Component, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub might: Attribute,
    pub fitness: Attribute,
    pub quickness: Attribute,
    pub intelligence: Attribute,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Quips {
    pub available: Vec<String>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SingleActivation;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EntityMoved;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct EntryTrigger;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Hidden;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct MagicMapper;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ProvidesFood;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Head,
    Torso,
    Legs,
    Feet,
    Hands,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute {
    Might,
    Quickness,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
    pub proc_chance: Option<f32>,
    pub proc_target: Option<String>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

impl Owned for Equipped {
    fn owned_by(&self) -> Entity {
        self.owner
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct DMSerializationHelper {
    pub map: MasterDungeonMap,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct SerializationHelper {
    pub map: Map,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Confusion;

#[derive(Component, ConvertSaveload, Clone)]
pub struct Duration {
    pub turns: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct StatusEffect {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Consumable {
    pub max_charges: i32,
    pub charges: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item: Entity,
}
#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

pub trait Owned {
    fn owned_by(&self) -> Entity;
}

impl Owned for InBackpack {
    fn owned_by(&self) -> Entity {
        self.owner
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Item {
    pub initiative_penalty: f32,
    pub weight_lbs: f32,
    pub base_value: f32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}
