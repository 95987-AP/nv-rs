//! A world for `world::crafting`: a workbench and a campfire category with
//! Aid and Ammo sub-categories, four recipes (one without a skill, one that
//! asks Science 40, one known only to those who hold its note, one at the
//! campfire), the items they use and the note.

use crate::{group, record, sub, zstr, TempData};

pub mod ids {
    pub const SCRAP: u32 = 0xC01;
    pub const WATER: u32 = 0xC02;
    pub const STEW: u32 = 0xC03;
    pub const BULLET: u32 = 0xC04;
    pub const NOTE: u32 = 0xC05;
    pub const BENCH: u32 = 0xC10;
    pub const FIRE: u32 = 0xC11;
    pub const AID: u32 = 0xC12;
    pub const AMMO: u32 = 0xC13;
    /// No skill: 2 scrap + 1 water make 1 stew (bench, Aid).
    pub const STEW_RECIPE: u32 = 0xC20;
    /// Science 40: 1 scrap makes 3 bullets (bench, Ammo).
    pub const BULLET_RECIPE: u32 = 0xC21;
    /// Shown only with the note: 1 water makes 1 stew (bench, Aid).
    pub const SECRET_RECIPE: u32 = 0xC22;
    /// At the campfire: 1 scrap makes 1 water (Aid).
    pub const FIRE_RECIPE: u32 = 0xC23;
}

/// The actor value number of Science (the script language's list).
pub const SCIENCE: u32 = 40;

/// The function number of `GetHasNote`.
const GET_HAS_NOTE: u16 = 382;

pub fn crafting(tag: &str) -> TempData {
    use ids::*;
    let dir = std::env::temp_dir().join(format!("nv-rs-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let data = TempData(dir);
    let named = |kind: &[u8; 4], id: u32, name: &str, rest: &[u8]| {
        let mut d = sub(b"EDID", &zstr(name));
        d.extend(rest);
        record(kind, id, &d)
    };
    let mut plugin = record(
        b"TES4",
        0,
        &sub(b"HEDR", &{
            let mut h = 1.34f32.to_le_bytes().to_vec();
            h.extend([0; 8]);
            h
        }),
    );
    let mut misc = named(b"MISC", SCRAP, "TestScrap", &sub(b"FULL", &zstr("Scrap")));
    misc.extend(named(
        b"MISC",
        WATER,
        "TestWater",
        &sub(b"FULL", &zstr("Water")),
    ));
    misc.extend(named(
        b"MISC",
        STEW,
        "TestStew",
        &sub(b"FULL", &zstr("Stew")),
    ));
    misc.extend(named(
        b"MISC",
        BULLET,
        "TestBullet",
        &sub(b"FULL", &zstr("Bullet")),
    ));
    plugin.extend(group(*b"MISC", 0, &misc));
    plugin.extend(group(
        *b"NOTE",
        0,
        &named(
            b"NOTE",
            NOTE,
            "TestRecipeNote",
            &sub(b"FULL", &zstr("Secret")),
        ),
    ));
    let category = |id: u32, edid: &str, name: &str| {
        let mut d = sub(b"FULL", &zstr(name));
        d.extend(sub(b"DATA", &[0]));
        named(b"RCCT", id, edid, &d)
    };
    let mut rcct = category(BENCH, "TestBench", "Workbench");
    rcct.extend(category(FIRE, "TestFire", "Campfire"));
    rcct.extend(category(AID, "TestAid", "Aid"));
    rcct.extend(category(AMMO, "TestAmmo", "Ammo"));
    plugin.extend(group(*b"RCCT", 0, &rcct));

    let recipe = |id: u32,
                  edid: &str,
                  name: &str,
                  skill: u32,
                  level: u32,
                  cat: u32,
                  subcat: u32,
                  inputs: &[(u32, i32)],
                  outputs: &[(u32, i32)],
                  condition: Option<Vec<u8>>| {
        let mut d = sub(b"FULL", &zstr(name));
        if let Some(c) = condition {
            d.extend(sub(b"CTDA", &c));
        }
        let mut dat = skill.to_le_bytes().to_vec();
        for v in [level, cat, subcat] {
            dat.extend(v.to_le_bytes());
        }
        d.extend(sub(b"DATA", &dat));
        for (item, n) in inputs {
            d.extend(sub(b"RCIL", &item.to_le_bytes()));
            d.extend(sub(b"RCQY", &n.to_le_bytes()));
        }
        for (item, n) in outputs {
            d.extend(sub(b"RCOD", &item.to_le_bytes()));
            d.extend(sub(b"RCQY", &n.to_le_bytes()));
        }
        named(b"RCPE", id, edid, &d)
    };
    // `GetHasNote Note == 1`.
    let has_note = {
        let mut c = vec![0u8; 4];
        c.extend(1.0f32.to_le_bytes());
        c.extend(GET_HAS_NOTE.to_le_bytes());
        c.extend([0, 0]);
        c.extend(NOTE.to_le_bytes());
        c.extend([0; 12]);
        c
    };
    let none = u32::MAX;
    let science = SCIENCE;
    let mut rcpe = recipe(
        STEW_RECIPE,
        "TestStewRecipe",
        "Stew",
        none,
        0,
        BENCH,
        AID,
        &[(SCRAP, 2), (WATER, 1)],
        &[(STEW, 1)],
        None,
    );
    rcpe.extend(recipe(
        BULLET_RECIPE,
        "TestBulletRecipe",
        "Bullets",
        science,
        40,
        BENCH,
        AMMO,
        &[(SCRAP, 1)],
        &[(BULLET, 3)],
        None,
    ));
    rcpe.extend(recipe(
        SECRET_RECIPE,
        "TestSecretRecipe",
        "Secret Stew",
        none,
        0,
        BENCH,
        AID,
        &[(WATER, 1)],
        &[(STEW, 1)],
        Some(has_note),
    ));
    rcpe.extend(recipe(
        FIRE_RECIPE,
        "TestFireRecipe",
        "Boiled Water",
        none,
        0,
        FIRE,
        AID,
        &[(SCRAP, 1)],
        &[(WATER, 1)],
        None,
    ));
    plugin.extend(group(*b"RCPE", 0, &rcpe));
    data.write("FalloutNV.esm", &plugin);
    data
}
