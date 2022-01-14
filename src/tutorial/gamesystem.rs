use crate::{Skill, Skills};

pub fn attr_bonus(value: i32) -> i32 {
    // see https://roll20.net/compendium/dnd5e/Ability%20Scores#content
    (value - 10) / 2
}

pub fn player_hp_per_level(fitness: i32) -> i32 {
    10 + attr_bonus(fitness)
}

pub fn player_hp_at_level(fitness: i32, level: i32) -> i32 {
    player_hp_per_level(fitness) * level
}

pub fn npc_hp(fitness: i32, level: i32) -> i32 {
    (0..level).fold(1, |sum, _| sum + i32::max(1, 8 + attr_bonus(fitness)))
}

pub fn mana_per_level(intelligence: i32) -> i32 {
    i32::max(1, 4 + attr_bonus(intelligence))
}

pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    mana_per_level(intelligence) * level
}

pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        skills.skills[&skill]
    } else {
        -4
    }
}
