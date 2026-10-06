//! Crafting: the recipes (`RCPE`) and categories (`RCCT`) the recipe menu
//! (`ShowRecipeMenu`, menu 1077) offers and what making one does.
//!
//! What the records hold (read from the data, not from the game's code):
//!
//! * a recipe's `DATA` is four numbers: the skill it asks for (an actor
//!   value number; `0xFFFFFFFF` for none), the level of it, its category
//!   (`RCCT`: Workbench, Campfire, Reloading Bench, the Sierra Madre
//!   Vending Machine ...) and its sub-category (`RCCT` again: Aid, Weapons,
//!   Ammo ...). `RCIL` + `RCQY` pairs are the ingredients, `RCOD` + `RCQY`
//!   pairs what it makes; `CTDA` conditions say when it is on offer. The
//!   learnt recipes' conditions ask `GetHasNote` for their recipe note, which
//!   the note's script (`AddNote`) gives when picked up.
//!
//! What nv-rs does with them is **not** read from the game's code yet (the
//! menu class and the make handler aren't traced), so each rule below says
//! it is a guess; `docs/DEAD_MONEY.md` "Crafting" lists them for checking
//! against the original game.

use esm::{FormId, FourCC, LoadOrder};

use crate::dialogue::{Condition, PLAYER_REF};
use crate::scripting::{Facts, GameState};

const RCPE: FourCC = FourCC::new(b"RCPE");
const RCCT: FourCC = FourCC::new(b"RCCT");
const DATA: FourCC = FourCC::new(b"DATA");
const RCIL: FourCC = FourCC::new(b"RCIL");
const RCOD: FourCC = FourCC::new(b"RCOD");
const RCQY: FourCC = FourCC::new(b"RCQY");
const CTDA: FourCC = FourCC::new(b"CTDA");

/// The misc statistic "Items Crafted" (number 32 in the game's table).
pub const ITEMS_CRAFTED: u8 = 32;

/// A recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    pub id: FormId,
    pub name: String,
    /// The actor value it asks for (`None`: no skill) and how much.
    pub skill: Option<u16>,
    pub level: u32,
    pub category: FormId,
    pub sub_category: FormId,
    /// What it uses up, and what it makes: (item, count).
    pub ingredients: Vec<(FormId, i32)>,
    pub outputs: Vec<(FormId, i32)>,
    pub conditions: Vec<Condition>,
}

impl Recipe {
    pub fn load(order: &LoadOrder, id: FormId) -> Option<Recipe> {
        let rr = order.get(id).filter(|r| r.entry.header.kind == RCPE)?;
        let record = rr.record().ok()?;
        let data = record.get(DATA).filter(|s| s.data.len() >= 16)?;
        let word = |at: usize| u32::from_le_bytes(data.data[at..at + 4].try_into().unwrap());
        let skill = word(0);
        let mut ingredients: Vec<(FormId, i32)> = Vec::new();
        let mut outputs: Vec<(FormId, i32)> = Vec::new();
        let mut conditions = Vec::new();
        // Each quantity belongs to the item before it.
        let mut last: Option<(bool, usize)> = None;
        for s in &record.subrecords {
            let form = |s: &esm::Subrecord| {
                (s.data.len() >= 4).then(|| {
                    rr.plugin
                        .to_global(FormId(u32::from_le_bytes(s.data[..4].try_into().unwrap())))
                })
            };
            match s.kind {
                k if k == RCIL => {
                    if let Some(f) = form(s) {
                        ingredients.push((f, 1));
                        last = Some((true, ingredients.len() - 1));
                    }
                }
                k if k == RCOD => {
                    if let Some(f) = form(s) {
                        outputs.push((f, 1));
                        last = Some((false, outputs.len() - 1));
                    }
                }
                k if k == RCQY && s.data.len() >= 4 => {
                    let n = i32::from_le_bytes(s.data[..4].try_into().unwrap());
                    match last {
                        Some((true, i)) => ingredients[i].1 = n,
                        Some((false, i)) => outputs[i].1 = n,
                        None => {}
                    }
                }
                k if k == CTDA => conditions.extend(crate::dialogue::read_condition(&rr, &s.data)),
                _ => {}
            }
        }
        Some(Recipe {
            id,
            name: record.full_name().unwrap_or_default(),
            skill: (skill != u32::MAX).then_some(skill as u16),
            level: if skill == u32::MAX { 0 } else { word(4) },
            category: rr.plugin.to_global(FormId(word(8))),
            sub_category: rr.plugin.to_global(FormId(word(12))),
            ingredients,
            outputs,
            conditions,
        })
    }
}

/// A category's (or sub-category's) name.
pub fn category_name(order: &LoadOrder, category: FormId) -> String {
    order
        .get(category)
        .filter(|r| r.entry.header.kind == RCCT)
        .and_then(|r| r.record().ok())
        .and_then(|r| r.full_name())
        .unwrap_or_default()
}

/// One line of the recipe menu.
#[derive(Debug, Clone, PartialEq)]
pub struct Offer {
    pub recipe: Recipe,
    /// Whether the player has what it asks: the skill and every ingredient.
    pub can_make: bool,
}

/// The skill's current value for the player (0 for a recipe with none).
pub fn skill_value(order: &LoadOrder, state: &GameState, recipe: &Recipe) -> f64 {
    let Some(av) = recipe.skill else {
        return 0.0;
    };
    Facts {
        order,
        state,
        speaker: None,
    }
    .current_actor_value(PLAYER_REF, av)
    .unwrap_or(0.0)
}

/// Whether the player can make a recipe now: the skill (a guess: the
/// player's current value, chems and all, at least the level) and every
/// ingredient in the quantity asked for.
pub fn can_make(order: &LoadOrder, state: &GameState, recipe: &Recipe) -> bool {
    skill_value(order, state, recipe) >= f64::from(recipe.level)
        && recipe
            .ingredients
            .iter()
            .all(|&(item, n)| state.item_count(order, PLAYER_REF, item) >= n)
}

/// The recipes the menu lists for a category (every category when none is
/// given: a guess), sorted by name: those whose conditions pass for the
/// player. Recipes the player can't make yet are listed too (a guess; the
/// menu shows them greyed).
pub fn offers(order: &LoadOrder, state: &GameState, category: Option<FormId>) -> Vec<Offer> {
    let facts = Facts {
        order,
        state,
        speaker: None,
    };
    let mut out: Vec<Offer> = order
        .records_of_type(RCPE)
        .filter_map(|rr| Recipe::load(order, rr.form_id))
        .filter(|r| category.map_or(true, |c| r.category == c))
        .filter(|r| facts.conditions_pass(&r.conditions, PLAYER_REF, PLAYER_REF))
        .map(|recipe| Offer {
            can_make: can_make(order, state, &recipe),
            recipe,
        })
        .collect();
    out.sort_by(|a, b| {
        a.recipe
            .name
            .to_ascii_lowercase()
            .cmp(&b.recipe.name.to_ascii_lowercase())
            .then(a.recipe.id.0.cmp(&b.recipe.id.0))
    });
    out
}

/// The sub-categories the offers fall in, in the order the first offer of
/// each appears, for the menu's filter arrows.
pub fn sub_categories(offers: &[Offer]) -> Vec<FormId> {
    let mut out: Vec<FormId> = Vec::new();
    for o in offers {
        if !out.contains(&o.recipe.sub_category) {
            out.push(o.recipe.sub_category);
        }
    }
    out
}

/// Why a recipe wasn't made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    NotARecipe,
    /// Its conditions don't pass: the player doesn't know it.
    Unknown,
    Skill,
    Ingredients,
}

/// Makes a recipe for the player: its ingredients are used up, its
/// outputs added, and "Items Crafted" counts one (a guess: one for each
/// time, whatever it makes). Returns what was made.
pub fn make(
    order: &LoadOrder,
    state: &mut GameState,
    recipe: FormId,
) -> Result<Vec<(FormId, i32)>, Refusal> {
    let recipe = Recipe::load(order, recipe).ok_or(Refusal::NotARecipe)?;
    let known = Facts {
        order,
        state,
        speaker: None,
    }
    .conditions_pass(&recipe.conditions, PLAYER_REF, PLAYER_REF);
    if !known {
        return Err(Refusal::Unknown);
    }
    if skill_value(order, state, &recipe) < f64::from(recipe.level) {
        return Err(Refusal::Skill);
    }
    if !recipe
        .ingredients
        .iter()
        .all(|&(item, n)| state.item_count(order, PLAYER_REF, item) >= n)
    {
        return Err(Refusal::Ingredients);
    }
    state.stock(order, PLAYER_REF);
    for &(item, n) in &recipe.ingredients {
        let have = state.items.get(&(PLAYER_REF, item)).copied().unwrap_or(0);
        if have <= n {
            state.items.remove(&(PLAYER_REF, item));
            state.unequip(PLAYER_REF, item);
        } else {
            state.items.insert((PLAYER_REF, item), have - n);
        }
    }
    for &(item, n) in &recipe.outputs {
        *state.items.entry((PLAYER_REF, item)).or_insert(0) += n;
    }
    crate::stats::bump(state, ITEMS_CRAFTED, 1);
    Ok(recipe.outputs)
}
