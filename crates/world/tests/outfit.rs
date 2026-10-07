//! What a person wears now (`world::outfit`), as the redraw follows it,
//! from a plugin built from scratch: their record's clothes from the
//! start, then equipping, taking off and taking away.

use esm::{ActivePlugins, FormId, LoadOrder};
use testdata::{group, record, sub, zstr};
use world::outfit::{start_worn, worn_armour};
use world::scripting::GameState;

const SHIRT: u32 = 0x900;
const DRESS: u32 = 0x901;
const HAT: u32 = 0x902;
const GLOVES_A: u32 = 0x903;
const GLOVES_B: u32 = 0x904;
const GLOVES: u32 = 0x950;
const NPC: u32 = 0x700;
const NPC_REF: u32 = 0x800;

fn plugin() -> Vec<u8> {
    let mut hedr = 1.34f32.to_le_bytes().to_vec();
    hedr.extend([0; 8]);
    let mut plugin = record(b"TES4", 0, &sub(b"HEDR", &hedr));
    let clothes = |id: u32, name: &str, slots: u32| {
        let mut d = sub(b"EDID", &zstr(name));
        let mut bmdt = slots.to_le_bytes().to_vec();
        bmdt.extend([0; 4]);
        d.extend(sub(b"BMDT", &bmdt));
        d.extend(sub(b"MODL", &zstr(&format!("Armor\\{name}.nif"))));
        record(b"ARMO", id, &d)
    };
    // A shirt (upper body), a dress over the upper body and the left hand,
    // a hat (the hat slot) and two pairs of gloves (both hands).
    let mut armour = clothes(SHIRT, "Shirt", 0x04);
    armour.extend(clothes(DRESS, "Dress", 0x04 | 0x08));
    armour.extend(clothes(HAT, "Hat", 0x400));
    armour.extend(clothes(GLOVES_A, "GlovesA", 0x08 | 0x10));
    armour.extend(clothes(GLOVES_B, "GlovesB", 0x08 | 0x10));
    plugin.extend(group(*b"ARMO", 0, &armour));
    // Gloves from a leveled list: either pair, at level 1.
    let mut list = sub(b"EDID", &zstr("LLGloves"));
    list.extend(sub(b"LVLD", &[0]));
    list.extend(sub(b"LVLF", &[0]));
    for gloves in [GLOVES_A, GLOVES_B] {
        let mut entry = 1u16.to_le_bytes().to_vec();
        entry.extend([0; 2]);
        entry.extend(gloves.to_le_bytes());
        entry.extend(1u16.to_le_bytes());
        entry.extend([0; 2]);
        list.extend(sub(b"LVLO", &entry));
    }
    plugin.extend(group(*b"LVLI", 0, &record(b"LVLI", GLOVES, &list)));
    // The person: carries the shirt, the hat and the gloves' list.
    let mut npc = sub(b"EDID", &zstr("TestPerson"));
    npc.extend(sub(b"MODL", &zstr("Characters\\_Male\\Skeleton.NIF")));
    npc.extend(sub(b"ACBS", &[0; 24]));
    for item in [SHIRT, HAT, GLOVES] {
        let mut cnto = item.to_le_bytes().to_vec();
        cnto.extend(1u32.to_le_bytes());
        npc.extend(sub(b"CNTO", &cnto));
    }
    plugin.extend(group(*b"NPC_", 0, &record(b"NPC_", NPC, &npc)));
    let placed = testdata::placed(NPC_REF, NPC, [0.0; 3], [0.0; 3], &[]);
    plugin.extend(group(*b"CELL", 0, &placed));
    plugin
}

fn ids(list: &[u32]) -> Vec<FormId> {
    list.iter().map(|&i| FormId(i)).collect()
}

#[test]
fn what_is_worn_follows_equipping_taking_off_and_taking_away() {
    let data = testdata::quests("outfit");
    data.write("FalloutNV.esm", &plugin());
    let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
    let who = FormId(NPC_REF);
    let start = start_worn(&order, FormId(NPC));
    // From the start: the shirt, the hat and the list's first gloves, as the
    // look draws them; the list's gloves are known to come from the list.
    assert_eq!(
        start.pieces.iter().map(|p| p.item).collect::<Vec<_>>(),
        ids(&[SHIRT, HAT, GLOVES_A])
    );
    assert_eq!(start.pieces[2].from, ids(&[GLOVES_A, GLOVES_B]));
    let mut state = GameState::new(&order);
    let worn = |state: &GameState| worn_armour(&order, state, who, &start);
    assert_eq!(worn(&state), ids(&[SHIRT, HAT, GLOVES_A]));
    // Untouched, the look made from what's worn is the record's own.
    let look = world::actor::npc_look_wearing(&order, FormId(NPC), &worn(&state), None);
    assert_eq!(look, world::actor_look(&order, FormId(NPC)));

    // Equipping what they don't carry does nothing (`0088c830`).
    state.equip(&order, who, FormId(DRESS));
    assert_eq!(worn(&state), ids(&[SHIRT, HAT, GLOVES_A]));
    // Given the dress, it's worn, and what it covers comes off.
    state.stock(&order, who);
    // The list's pick, whichever it was, keeps the gloves drawn.
    state.items.remove(&(who, FormId(GLOVES_B)));
    state.items.insert((who, FormId(GLOVES_A)), 1);
    state.items.insert((who, FormId(DRESS)), 1);
    state.equip(&order, who, FormId(DRESS));
    assert_eq!(worn(&state), ids(&[HAT, DRESS]));
    let models: Vec<String> =
        world::actor::npc_look_wearing(&order, FormId(NPC), &worn(&state), None)
            .unwrap()
            .parts
            .into_iter()
            .map(|p| p.model)
            .collect();
    assert_eq!(models, ["Armor\\Hat.nif", "Armor\\Dress.nif"]);
    // Taken off: what it replaced stays off while they still carry it.
    state.unequip_item(&order, who, FormId(DRESS));
    assert_eq!(worn(&state), ids(&[HAT]));
    // Taken away: they pick again, and wear the shirt and gloves again.
    state.items.remove(&(who, FormId(DRESS)));
    assert_eq!(worn(&state), ids(&[SHIRT, HAT, GLOVES_A]));

    // The hat taken off, then put on again.
    state.unequip_item(&order, who, FormId(HAT));
    assert_eq!(worn(&state), ids(&[SHIRT, GLOVES_A]));
    state.equip(&order, who, FormId(HAT));
    assert_eq!(worn(&state), ids(&[SHIRT, GLOVES_A, HAT]));

    // Taken away from the start's: the shirt; the gloves once neither
    // pair the list could give is left.
    state.items.remove(&(who, FormId(SHIRT)));
    assert_eq!(worn(&state), ids(&[GLOVES_A, HAT]));
    state.items.insert((who, FormId(GLOVES_A)), 0);
    state.items.insert((who, FormId(GLOVES_B)), 1);
    assert_eq!(worn(&state), ids(&[GLOVES_A, HAT]));
    state.items.remove(&(who, FormId(GLOVES_B)));
    assert_eq!(worn(&state), ids(&[HAT]));
}

#[test]
fn what_came_off_is_saved() {
    let data = testdata::quests("outfit-save");
    data.write("FalloutNV.esm", &plugin());
    let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
    let who = FormId(NPC_REF);
    let mut state = GameState::new(&order);
    state.stock(&order, who);
    state.items.insert((who, FormId(DRESS)), 1);
    state.equip(&order, who, FormId(DRESS));
    state.unequip_item(&order, who, FormId(HAT));
    let off = state.taken_off.get(&who).cloned().unwrap();
    assert!(off.contains(&(FormId(SHIRT), Some(FormId(DRESS)))));
    assert!(off.contains(&(FormId(HAT), None)));
    let text = world::save::save(&state, None);
    let (loaded, _) = world::save::load(&text).unwrap();
    assert_eq!(loaded.taken_off.get(&who), Some(&off));
}
