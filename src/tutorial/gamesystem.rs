pub fn attr_bonus(value: i32) -> i32 {
    // see https://roll20.net/compendium/dnd5e/Ability%20Scores#content
    (value - 10) / 2
}
