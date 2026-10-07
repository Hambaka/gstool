/// Valid psynergy items in TBS
///
/// C8: Orb of Force
/// C9: Douse Drop
/// CA: Frost Jewel
/// CB: Lifting Gem
/// CC: Halt Gem
/// CD: Cloak Ball
/// CE: Carry Stone
/// CF: Catch Beads
pub const PSYNERGY_ITEMS: [u16; 8] = [0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF];

/// Valid stackable items in TBS
///
/// B4: Herb
/// B5: Nut
/// B6: Vial
/// B7: Potion
/// BA: Psy Crystal
/// BB: Antidote
/// BC: Elixir
/// BD: Water of Life
/// BF: Power Bread
/// C0: Cookie
/// C1: Apple
/// C2: Hard Nut
/// C3: Mint
/// C4: Lucky Pepper
/// E2: Smoke Bomb
/// E3: Sleep Bomb
/// E4: Game Ticket
/// E5: Lucky Medal
/// EC: Sacred Feather
/// EE: Oil Drop
/// EF: Weasel's Claw
/// F0: Bramble Seed
/// F1: Crystal Powder
pub const STACKABLE_ITEMS: [u16; 23] = [
  0xB4, 0xB5, 0xB6, 0xB7, 0xBA, 0xBB, 0xBC, 0xBD, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xE2, 0xE3,
  0xE4, 0xE5, 0xEC, 0xEE, 0xEF, 0xF0, 0xF1,
];
