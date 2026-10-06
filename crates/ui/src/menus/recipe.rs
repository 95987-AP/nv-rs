//! Crafting (`menus\recipe_menu.xml`, class `RecipeMenu` 1077, vtable
//! `0107048c` in FalloutNV.exe): the recipes of a category on the left, the
//! one under the pointer's ingredients, skill and category on the right,
//! Accept to make it. Read from the code (`00726ff0` and the functions
//! after it; names from the Xbox 360 prototype's symbols, Xbox PDB). The
//! rules (what is listed, how many can be made, making) are
//! `world::crafting`'s; this is the menu's side:
//!
//! * opening (`RecipeMenu::Create` (Xbox PDB), `00726ff0`): list boxes on
//!   ids 3 and 6 (the code asks for `CM_list_template`, which the file lacks,
//!   so the lookup ends on its `RM_list_template`); the title (1)
//!   `sRecipes`, 13 `sSkillRequirement`, 11 `sIngredients`, 12 `sMadeAt`,
//!   Accept (7) `sAccept`, Exit (8) `sExit`, the two buttons then 20 units
//!   further left; then filled.
//! * filling (`00727680`): one line (template id 15) per recipe in the
//!   order given, its alpha 255, or 127 when none can be made; the filter's
//!   subcategories.
//! * the filter (clicks on 0, 1 and 2, `007274b0`; the arrows, special
//!   codes 4 and 3, `00728420`): the index (`0119fbf4`, -1 for all, kept
//!   from one opening to the next) steps down on 0 and up on 1 and 2,
//!   wrapping through -1; the title shows `sRecipes` or the subcategory's
//!   name; lines of other subcategories are filtered out (`00728d60`).
//! * the pointer on a recipe (`00727b10` with id 15): the item picture and
//!   card (9, 10) hidden, then the caller's [`Details`]: the category (4,
//!   alpha 127 when it isn't the menu's), the ingredients heading (11), the
//!   ingredient lines (6, alpha 127 when short), the skill (5, alpha 127
//!   when too low), Accept's `target` (only when it can be made) and the
//!   first product's picture (9).
//! * Accept (7, `007274b0` case 7): the quantity menu for how many (at most
//!   as many as can be made), then the making (`007284f0`), which closes
//!   the menu; Exit (8): closes it (`00727430`).
//!
//! Not here: the item card (`RecipeMenu::PopulateItemStatsDisplay`,
//! `00728da0`), the ingredient list's own pointer (an ingredient's
//! picture), the controller's list switching (special 0xd, 0xe) and the
//! accept button's cross-fade (`00728a70`).

use crate::list::ListBox;
use crate::menu::{self, MenuCode};
use crate::names::t;
use crate::tile::{TileId, Ui};

/// The menu's file.
pub const FILE: &str = "menus\\recipe_menu.xml";
/// Its class number (`00726f00`).
pub const CLASS: i32 = 1077;
/// A list line's id (the template's).
pub const LINE_ID: i32 = 15;
/// The template the code asks for.
const TEMPLATE: &str = "CM_list_template";

const LEFT_ARROW: usize = 0;
const TITLE: usize = 1;
const RIGHT_ARROW: usize = 2;
const RECIPES: usize = 3;
const MADE_AT: usize = 4;
const SKILL: usize = 5;
const INGREDIENTS: usize = 6;
const ACCEPT: usize = 7;
const EXIT: usize = 8;
const ICON: usize = 9;
const CARD: usize = 10;
const INGREDIENTS_TITLE: usize = 11;
const MADE_AT_TITLE: usize = 12;
const SKILL_TITLE: usize = 13;

/// The alphas the code gives lines and fields (`00700320(0xfa9, ..)`).
const BRIGHT: f32 = 255.0;
const DIM: f32 = 127.0;

/// A recipe line: its text ("name" or "name (n)"), whether it can be made
/// at least once, and its subcategory (a form ID) for the filter.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub text: String,
    pub makeable: bool,
    pub subcategory: u32,
}

/// What the right side shows for the recipe under the pointer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Details {
    /// The recipe's category name, and whether it is dimmed (not the
    /// menu's category).
    pub made_at: String,
    pub made_at_dim: bool,
    /// For a recipe of one ingredient and several products, its
    /// subcategory's name (its products are listed then); `None` for
    /// `sIngredients`.
    pub heading: Option<String>,
    /// The ingredient (or product) lines: text, and whether dimmed.
    pub parts: Vec<(String, bool)>,
    /// "skill (have/need)", empty for none, and whether dimmed.
    pub skill: String,
    pub skill_dim: bool,
    /// Accept can be used.
    pub can_make: bool,
    /// The first product's picture.
    pub icon: Option<String>,
}

/// What the menu asks of the game.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// The pointer is on this line (its index): give its details.
    Show(usize),
    /// Accept on this line: ask how many and make them.
    Make(usize),
    /// Exit.
    Close,
}

#[derive(Debug)]
pub struct RecipeMenu {
    pub menu: TileId,
    pub tiles: [Option<TileId>; 14],
    /// The recipes (`+0x6c`) and the ingredients (`+0xa0`).
    pub recipes: ListBox,
    pub parts: ListBox,
    pub lines: Vec<Line>,
    /// The filter's subcategories: form ID and name.
    pub filters: Vec<(u32, String)>,
    /// The filter's index (`0119fbf4`): -1 for all.
    pub filter: i32,
    /// The line whose details are shown (`011d8e94`).
    pub shown: Option<usize>,
    pub requests: Vec<Request>,
    pub closed: bool,
}

fn setting(ui: &Ui, name: &str) -> String {
    ui.setting_text(name).unwrap_or_default()
}

/// A line's alpha goes on its first child (`007b5370` reads the new
/// line's child list; the template's first child is `ListItemText`).
fn line_alpha(ui: &mut Ui, line: TileId, dim: bool) {
    let text = ui.tiles[line]
        .children
        .first()
        .copied()
        .or_else(|| ui.find_below(line, "ListItemText"));
    if let Some(text) = text {
        ui.set_number(text, t::ALPHA, if dim { DIM } else { BRIGHT });
    }
}

impl RecipeMenu {
    /// A menu whose filter starts at `filter` (what it was when the menu
    /// last closed; -1 the first time).
    pub fn new(menu: TileId, filter: i32) -> RecipeMenu {
        RecipeMenu {
            menu,
            tiles: [None; 14],
            recipes: ListBox::default(),
            parts: ListBox::default(),
            lines: Vec::new(),
            filters: Vec::new(),
            filter,
            shown: None,
            requests: Vec::new(),
            closed: false,
        }
    }

    fn tile(&self, id: usize) -> Option<TileId> {
        self.tiles.get(id).copied().flatten()
    }

    fn set_text(&self, ui: &mut Ui, id: usize, text: &str) {
        if let Some(tile) = self.tile(id) {
            ui.set_text(tile, t::STRING, text);
        }
    }

    fn set(&self, ui: &mut Ui, id: usize, trait_id: i32, value: f32) {
        if let Some(tile) = self.tile(id) {
            ui.set_number(tile, trait_id, value);
        }
    }

    /// Opens it (`00726ff0`) with the lines (in the order to show them) and
    /// the filter's subcategories; false without its lists.
    pub fn open(&mut self, ui: &mut Ui, lines: Vec<Line>, filters: Vec<(u32, String)>) -> bool {
        let (Some(recipes), Some(parts)) = (self.tile(RECIPES), self.tile(INGREDIENTS)) else {
            return false;
        };
        self.recipes = ListBox::new(ui, recipes, TEMPLATE);
        self.parts = ListBox::new(ui, parts, TEMPLATE);
        for (id, name) in [
            (TITLE, "sRecipes"),
            (SKILL_TITLE, "sSkillRequirement"),
            (INGREDIENTS_TITLE, "sIngredients"),
            (MADE_AT_TITLE, "sMadeAt"),
            (ACCEPT, "sAccept"),
            (EXIT, "sExit"),
        ] {
            let s = setting(ui, name);
            self.set_text(ui, id, &s);
        }
        for id in [ACCEPT, EXIT] {
            if let Some(tile) = self.tile(id) {
                let x = ui.number(tile, t::X);
                ui.set_number(tile, t::X, x - 20.0);
            }
        }
        self.lines = lines;
        self.filters = filters;
        if self.filter >= self.filters.len() as i32 {
            self.filter = -1;
        }
        for (i, line) in self.lines.iter().enumerate() {
            if let Some(tile) = self.recipes.add(ui, self.menu, i as i32, Some(&line.text)) {
                line_alpha(ui, tile, !line.makeable);
            }
        }
        self.apply_filter(ui);
        ui.set_number(self.menu, menu::LEAVE_STACK, 0.0);
        ui.set_number(self.menu, t::VISIBLE, 1.0);
        ui.refresh();
        true
    }

    /// The title and the filtered lines for the filter's index
    /// (`007274b0` cases 0 to 2, `00728d60`).
    fn apply_filter(&mut self, ui: &mut Ui) {
        let title = match usize::try_from(self.filter)
            .ok()
            .and_then(|i| self.filters.get(i))
        {
            Some((_, name)) => name.clone(),
            None => setting(ui, "sRecipes"),
        };
        self.set_text(ui, TITLE, &title);
        let only = usize::try_from(self.filter)
            .ok()
            .and_then(|i| self.filters.get(i))
            .map(|(id, _)| *id);
        let lines = self.lines.clone();
        self.recipes.filter(ui, &|value| {
            only.is_some_and(|sub| {
                lines
                    .get(value as usize)
                    .is_some_and(|l| l.subcategory != sub)
            })
        });
        ui.refresh();
    }

    fn step_filter(&mut self, ui: &mut Ui, up: bool) {
        let n = self.filters.len() as i32;
        self.filter += if up { 1 } else { -1 };
        if self.filter < -1 {
            self.filter = n - 1;
        } else if self.filter >= n {
            self.filter = -1;
        }
        self.apply_filter(ui);
    }

    /// Shows the details of the line under the pointer (`00727b10`).
    pub fn show_details(&mut self, ui: &mut Ui, line: usize, d: &Details) {
        self.shown = Some(line);
        self.set(ui, CARD, t::VISIBLE, 0.0);
        self.set(ui, ICON, t::VISIBLE, 0.0);
        self.set_text(ui, MADE_AT, &d.made_at);
        self.set(
            ui,
            MADE_AT,
            t::ALPHA,
            if d.made_at_dim { DIM } else { BRIGHT },
        );
        let heading = d
            .heading
            .clone()
            .unwrap_or_else(|| setting(ui, "sIngredients"));
        self.set_text(ui, INGREDIENTS_TITLE, &heading);
        self.parts.clear(ui);
        for (i, (text, dim)) in d.parts.iter().enumerate() {
            if let Some(tile) = self.parts.add(ui, self.menu, i as i32, Some(text)) {
                line_alpha(ui, tile, *dim);
            }
        }
        self.set_text(ui, SKILL, &d.skill);
        self.set(ui, SKILL, t::ALPHA, if d.skill_dim { DIM } else { BRIGHT });
        self.set(ui, ACCEPT, t::TARGET, if d.can_make { 1.0 } else { 0.0 });
        if let (Some(icon), Some(tile)) = (&d.icon, self.tile(ICON)) {
            ui.set_string(tile, t::FILENAME, icon);
            ui.set_number(tile, t::VISIBLE, 1.0);
        }
        ui.refresh();
    }

    /// Exit, and the end of making (`00727430`).
    pub fn close(&mut self, ui: &mut Ui) {
        if self.closed {
            return;
        }
        ui.set_number(self.menu, menu::LEAVE_STACK, 1.0);
        self.closed = true;
    }
}

impl MenuCode for RecipeMenu {
    fn class(&self) -> i32 {
        CLASS
    }

    fn set_tile(&mut self, id: i32, tile: TileId) {
        if (0..14).contains(&id) {
            self.tiles[id as usize] = Some(tile);
        }
    }

    /// The recipes and the ingredients: the pointer chooses lines in them
    /// (`00717e70`).
    fn lists(&mut self) -> Vec<&mut ListBox> {
        vec![&mut self.recipes, &mut self.parts]
    }

    /// `007274b0`.
    fn click(&mut self, ui: &mut Ui, id: i32, _tile: Option<TileId>, _now: f64) {
        match id {
            0 => self.step_filter(ui, false),
            1 | 2 => self.step_filter(ui, true),
            7 => {
                let usable = self
                    .tile(ACCEPT)
                    .is_some_and(|tile| ui.number(tile, t::TARGET) != 0.0);
                if let Some(line) = self.shown.filter(|_| usable) {
                    self.requests.push(Request::Make(line));
                }
            }
            8 => {
                self.requests.push(Request::Close);
                self.close(ui);
            }
            _ => {}
        }
    }

    /// `00727b10` with a recipe line.
    fn mouseover(&mut self, ui: &mut Ui, id: i32, tile: TileId) {
        if id != LINE_ID {
            return;
        }
        if let Some(value) = self
            .recipes
            .items
            .iter()
            .find(|i| i.tile == tile)
            .map(|i| i.value)
        {
            let _ = ui;
            self.requests.push(Request::Show(value as usize));
        }
    }

    /// `00728420`: Left and Right step the filter.
    fn special_key(&mut self, ui: &mut Ui, code: i32, now: f64) -> bool {
        match code {
            menu::special::RIGHT => {
                self.click(ui, RIGHT_ARROW as i32, None, now);
                true
            }
            menu::special::LEFT => {
                self.click(ui, LEFT_ARROW as i32, None, now);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menus::test_support;

    fn line(text: &str, makeable: bool, subcategory: u32) -> Line {
        Line {
            text: text.into(),
            makeable,
            subcategory,
        }
    }

    fn opened(filter: i32) -> (Ui, RecipeMenu) {
        let mut ui = test_support::ui();
        let mut m = RecipeMenu::new(0, filter);
        m.menu = test_support::load(&mut ui, &test_support::recipe_menu(), &mut m);
        let lines = vec![
            line("Healing Powder", true, 0x19),
            line("Rounds (10)", true, 0x2A),
            line("Stimpak", false, 0x19),
        ];
        let filters = vec![(0x19, "Aid".to_string()), (0x2A, "Ammo".to_string())];
        assert!(m.open(&mut ui, lines, filters));
        (ui, m)
    }

    fn shown(ui: &mut Ui, m: &RecipeMenu) -> Vec<(String, f32)> {
        m.recipes
            .shown_items(ui, 0, i32::MAX)
            .into_iter()
            .map(|l| {
                (
                    ui.string(l.tile, t::STRING).unwrap_or_default(),
                    ui.number(ui.tiles[l.tile].children[0], t::ALPHA),
                )
            })
            .collect()
    }

    /// `00726ff0`, `00727680`, `007274b0`: the settings, the lines with
    /// their alphas, and the filter stepping through -1.
    #[test]
    fn opening_and_the_filter() {
        let (mut ui, mut m) = opened(-1);
        let s = |ui: &mut Ui, m: &RecipeMenu, id: usize| {
            ui.string(m.tiles[id].unwrap(), t::STRING)
                .unwrap_or_default()
        };
        assert_eq!(s(&mut ui, &m, TITLE), "RECIPES");
        assert_eq!(s(&mut ui, &m, ACCEPT), "Accept");
        assert_eq!(s(&mut ui, &m, EXIT), "Exit");
        assert_eq!(
            shown(&mut ui, &m),
            [
                ("Healing Powder".to_string(), 255.0),
                ("Rounds (10)".to_string(), 255.0),
                ("Stimpak".to_string(), 127.0)
            ]
        );
        // Up: Aid, then Ammo, then all again.
        m.click(&mut ui, 2, None, 0.0);
        assert_eq!(s(&mut ui, &m, TITLE), "Aid");
        assert_eq!(shown(&mut ui, &m).len(), 2);
        m.click(&mut ui, 1, None, 0.0);
        assert_eq!(s(&mut ui, &m, TITLE), "Ammo");
        assert_eq!(shown(&mut ui, &m)[0].0, "Rounds (10)");
        m.click(&mut ui, 2, None, 0.0);
        assert_eq!(m.filter, -1);
        assert_eq!(shown(&mut ui, &m).len(), 3);
        // Down from all: the last subcategory.
        m.click(&mut ui, 0, None, 0.0);
        assert_eq!(s(&mut ui, &m, TITLE), "Ammo");
        // Kept: a new menu starts where it was.
        let (mut ui, m) = opened(1);
        assert_eq!(
            ui.string(m.tiles[TITLE].unwrap(), t::STRING).as_deref(),
            Some("Ammo")
        );
    }

    /// `00727b10`, `007274b0` cases 7 and 8: details, Accept only when it
    /// can be made, Exit closes.
    #[test]
    fn details_accept_and_exit() {
        let (mut ui, mut m) = opened(-1);
        let first = m.recipes.items[0].tile;
        m.mouseover(&mut ui, LINE_ID, first);
        assert_eq!(m.requests, [Request::Show(0)]);
        let d = Details {
            made_at: "Campfire".into(),
            made_at_dim: false,
            heading: None,
            parts: vec![
                ("Broc Flower (1/1)".into(), false),
                ("Xander Root (0/1)".into(), true),
            ],
            skill: String::new(),
            skill_dim: false,
            can_make: false,
            icon: Some("Interface\\Icons\\Items\\healingpowder.dds".into()),
        };
        m.show_details(&mut ui, 0, &d);
        let parts: Vec<(String, f32)> = m
            .parts
            .shown_items(&mut ui, 0, i32::MAX)
            .into_iter()
            .map(|l| {
                (
                    ui.string(l.tile, t::STRING).unwrap_or_default(),
                    ui.number(ui.tiles[l.tile].children[0], t::ALPHA),
                )
            })
            .collect();
        assert_eq!(
            parts,
            [
                ("Broc Flower (1/1)".to_string(), 255.0),
                ("Xander Root (0/1)".to_string(), 127.0)
            ]
        );
        assert_eq!(ui.number(m.tiles[ICON].unwrap(), t::VISIBLE), 1.0);
        // Accept does nothing while it can't be made.
        m.requests.clear();
        m.click(&mut ui, 7, None, 0.0);
        assert!(m.requests.is_empty());
        m.show_details(
            &mut ui,
            0,
            &Details {
                can_make: true,
                ..d
            },
        );
        m.click(&mut ui, 7, None, 0.0);
        assert_eq!(m.requests, [Request::Make(0)]);
        m.click(&mut ui, 8, None, 0.0);
        assert!(m.closed);
        assert_eq!(ui.number(m.menu, menu::LEAVE_STACK), 1.0);
    }
}
