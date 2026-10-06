//! The recipe menu (`menus\recipe_menu.xml`, class `RecipeMenu` 1077),
//! opened by a script's `ShowRecipeMenu` (a workbench, a campfire, a
//! reloading bench, the Sierra Madre vending machines).
//!
//! What the file lays out, read from it: the recipes in a list box (id 3,
//! rows of the file's `RM_list_template`, id 15) under a title (id 1, `sRecipes`)
//! with filter arrows (ids 0 and 2); on the right "Made at ..." (id 4), the
//! skill requirement (id 5) and the ingredients' list box (id 6); Accept
//! (id 7, key A) and Exit (id 8, key X); the recipe's picture (id 9).
//!
//! What this port does with it is **not** read from the menu class's code
//! (`00726ff0` and the functions after it aren't traced yet): every rule
//! below that isn't the file's own is a guess, listed in
//! `docs/DEAD_MONEY.md` "Crafting": a row under the pointer is chosen
//! (the interface does that for every list box) and shows its details;
//! Accept (or A) makes the chosen recipe when its skill and ingredients are
//! there, and the button is off (`target` 0, which answers a key with the
//! cancel sound) otherwise; the arrows (and Left / Right) go through the
//! sub-categories with all of them first; a recipe that can't be made yet
//! has the "failed check" look.

use crate::list::ListBox;
use crate::menu::{self, MenuCode};
use crate::names::t;
use crate::tile::{TileId, Ui};

/// The menu's file.
pub const FILE: &str = "menus\\recipe_menu.xml";
/// Its class number.
pub const CLASS: i32 = 1077;
/// The id of the list's rows (the template's).
pub const ROW_ID: i32 = 15;
/// The recipes' template, and the ingredients' (the same one).
const TEMPLATE: &str = "RM_list_template";
const TILE_COUNT: usize = 16;

/// One ingredient of a recipe: its name, how many the player has and the
/// recipe needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Ingredient {
    pub name: String,
    pub have: i32,
    pub need: i32,
}

/// One recipe as the menu shows it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecipeLine {
    /// The recipe's form.
    pub form: u32,
    pub name: String,
    /// Its sub-category's form (see [`Filter`]).
    pub sub: u32,
    /// Whether the player has the skill and every ingredient.
    pub makeable: bool,
    /// "Science 40", empty for none.
    pub skill: String,
    pub ingredients: Vec<Ingredient>,
    /// The picture of what it makes.
    pub icon: String,
}

/// A sub-category the filter arrows go to.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub form: u32,
    pub name: String,
}

/// What the menu asks of its caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Make this recipe (Accept).
    Make(u32),
}

/// The recipe menu.
#[derive(Debug)]
pub struct RecipeMenu {
    pub menu: TileId,
    /// `SetTile` ids 0..16 (`RM_Items_LeftFilterArrow` 0, `RM_ItemsTitle`
    /// 1, the right arrow 2, the recipes' list 3, `RM_MadeAtVariable` 4,
    /// `RM_SkillRequirement` 5, the ingredients' list 6, Accept 7, Exit 8,
    /// `RM_ItemIcon` 9, `RM_ItemData` 10).
    pub tiles: [Option<TileId>; TILE_COUNT],
    /// The recipes' list and the ingredients'.
    pub lists: [ListBox; 2],
    pub lines: Vec<RecipeLine>,
    pub filters: Vec<Filter>,
    /// The filter (0 all, then `filters` in turn).
    pub filter: usize,
    /// "Workbench", ...: the category's name.
    pub made_at: String,
    /// The recipe chosen (its index in `lines`).
    pub chosen: Option<usize>,
    pub requests: Vec<Request>,
    pub sounds: Vec<String>,
    pub closed: bool,
}

impl RecipeMenu {
    pub fn new(menu: TileId) -> RecipeMenu {
        RecipeMenu {
            menu,
            tiles: [None; TILE_COUNT],
            lists: [ListBox::default(), ListBox::default()],
            lines: Vec::new(),
            filters: Vec::new(),
            filter: 0,
            made_at: String::new(),
            chosen: None,
            requests: Vec::new(),
            sounds: Vec::new(),
            closed: false,
        }
    }

    fn tile(&self, id: usize) -> Option<TileId> {
        self.tiles.get(id).copied().flatten()
    }

    /// Opens it; false when the file lacks one of its tiles.
    pub fn open(
        &mut self,
        ui: &mut Ui,
        made_at: &str,
        lines: Vec<RecipeLine>,
        filters: Vec<Filter>,
    ) -> bool {
        if (0..=9).any(|i| self.tile(i).is_none()) {
            return false;
        }
        for (i, id) in [(0, 3), (1, 6)] {
            if let Some(list) = self.tile(id) {
                self.lists[i] = ListBox::new(ui, list, TEMPLATE);
            }
        }
        self.made_at = made_at.to_string();
        self.lines = lines;
        self.filters = filters;
        self.filter = 0;
        self.chosen = None;
        ui.set_number(self.menu, menu::LEAVE_STACK, 0.0);
        ui.set_number(self.menu, t::VISIBLE, 1.0);
        self.fill(ui);
        true
    }

    /// Replaces the recipes after one was made (what the player has
    /// changed); the choice stays on the same recipe when it's still there.
    pub fn refreshed(&mut self, ui: &mut Ui, lines: Vec<RecipeLine>) {
        let keep = self.chosen.and_then(|i| self.lines.get(i)).map(|l| l.form);
        self.lines = lines;
        self.fill(ui);
        if let Some(form) = keep {
            self.choose_form(ui, form);
        }
    }

    fn custom(ui: &mut Ui, name: &str) -> i32 {
        ui.names.lookup_or_add(name).unwrap_or(0)
    }

    /// The lines the filter lets through (indexes into `lines`).
    fn shown(&self) -> Vec<usize> {
        let sub = match self.filter {
            0 => None,
            n => self.filters.get(n - 1).map(|f| f.form),
        };
        (0..self.lines.len())
            .filter(|&i| sub.map_or(true, |s| self.lines[i].sub == s))
            .collect()
    }

    /// Fills the recipes' list for the filter, the title with the filter's
    /// name, and clears the details.
    fn fill(&mut self, ui: &mut Ui) {
        let shown = self.shown();
        self.lists[0].clear(ui);
        for i in shown {
            let (name, makeable) = (self.lines[i].name.clone(), self.lines[i].makeable);
            if let Some(tile) = self.lists[0].add(ui, self.menu, i as i32, Some(&name)) {
                ui.tiles[tile].failed = !makeable;
            }
        }
        if let Some(title) = self.tile(1) {
            let s = match self.filter {
                0 => ui.setting_text("sRecipes").unwrap_or_default(),
                n => self
                    .filters
                    .get(n - 1)
                    .map(|f| f.name.to_uppercase())
                    .unwrap_or_default(),
            };
            ui.set_text(title, t::STRING, &s);
        }
        self.chosen = None;
        self.details(ui);
    }

    /// Chooses the row of a recipe, if it's listed.
    fn choose_form(&mut self, ui: &mut Ui, form: u32) {
        let Some(index) = self.lines.iter().position(|l| l.form == form) else {
            return;
        };
        let row = self.lists[0]
            .items
            .iter()
            .find(|i| i.value == index as i32)
            .map(|i| i.tile);
        if let Some(row) = row {
            self.lists[0].select(ui, Some(row));
            self.chosen = Some(index);
            self.details(ui);
        }
    }

    /// Shows the chosen recipe's details (or none): where it's made, the
    /// skill, the ingredients, the picture; Accept on or off.
    fn details(&mut self, ui: &mut Ui) {
        let line = self.chosen.and_then(|i| self.lines.get(i)).cloned();
        let made_at = if line.is_some() {
            self.made_at.as_str()
        } else {
            ""
        };
        if let Some(tile) = self.tile(4) {
            ui.set_text(tile, t::STRING, made_at);
        }
        if let Some(tile) = self.tile(5) {
            let s = line.as_ref().map(|l| l.skill.as_str()).unwrap_or("");
            ui.set_text(tile, t::STRING, s);
        }
        self.lists[1].clear(ui);
        let value = Self::custom(ui, "_Value");
        if let Some(line) = &line {
            for ing in &line.ingredients {
                if let Some(row) = self.lists[1].add(ui, self.menu, 0, Some(&ing.name)) {
                    ui.set_string(row, value, &format!("{}/{}", ing.have, ing.need));
                    ui.tiles[row].failed = ing.have < ing.need;
                }
            }
        }
        if let Some(icon) = self.tile(9) {
            match line.as_ref().filter(|l| !l.icon.is_empty()) {
                Some(l) => {
                    ui.set_string(icon, t::FILENAME, &l.icon);
                    ui.set_number(icon, t::VISIBLE, 1.0);
                }
                None => ui.set_number(icon, t::VISIBLE, 0.0),
            }
        }
        if let Some(accept) = self.tile(7) {
            let on = line.as_ref().is_some_and(|l| l.makeable);
            ui.set_number(accept, t::TARGET, f32::from(on));
        }
        ui.refresh();
    }

    /// Steps the filter by one either way, wrapping.
    fn step_filter(&mut self, ui: &mut Ui, by: i32) {
        let n = self.filters.len() as i32 + 1;
        self.filter = (self.filter as i32 + by).rem_euclid(n) as usize;
        if let Some(s) = menu::menu_sound(3) {
            self.sounds.push(s.to_string());
        }
        self.fill(ui);
    }

    fn close(&mut self, ui: &mut Ui) {
        ui.set_number(self.menu, menu::LEAVE_STACK, 1.0);
        self.closed = true;
    }
}

impl MenuCode for RecipeMenu {
    fn class(&self) -> i32 {
        CLASS
    }

    fn set_tile(&mut self, id: i32, tile: TileId) {
        if (0..TILE_COUNT as i32).contains(&id) {
            self.tiles[id as usize] = Some(tile);
        }
    }

    fn click(&mut self, ui: &mut Ui, id: i32, _tile: Option<TileId>, _now: f64) {
        match id {
            0 => self.step_filter(ui, -1),
            2 => self.step_filter(ui, 1),
            7 => {
                if let Some(line) = self.chosen.and_then(|i| self.lines.get(i)) {
                    if line.makeable {
                        self.requests.push(Request::Make(line.form));
                    }
                }
            }
            8 => self.close(ui),
            // A row clicked: it was chosen under the pointer; the click
            // makes it (a guess; the buttons do the same).
            ROW_ID => self.click(ui, 7, None, 0.0),
            _ => {}
        }
    }

    fn mouseover(&mut self, ui: &mut Ui, id: i32, _tile: TileId) {
        if id != ROW_ID {
            return;
        }
        let list = &self.lists[0];
        let chosen = list
            .selected
            .and_then(|s| list.value_of(s))
            .map(|v| v as usize);
        if chosen.is_some() && chosen != self.chosen {
            self.chosen = chosen;
            self.details(ui);
        }
    }

    fn special_key(&mut self, ui: &mut Ui, code: i32, _now: f64) -> bool {
        match code {
            menu::special::LEFT => self.click(ui, 0, None, 0.0),
            menu::special::RIGHT => self.click(ui, 2, None, 0.0),
            _ => return false,
        }
        true
    }

    fn lists(&mut self) -> Vec<&mut ListBox> {
        self.lists.iter_mut().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menus::test_support::{self, list_box, LIST_ITEM};

    /// A recipe menu laid out like `recipe_menu.xml`.
    fn recipe_menu() -> String {
        format!(
            "<menu name=\"RecipeMenu\"><class>&RecipeMenu;</class>
               <_PCButton_A>RM_ButtonX</_PCButton_A><_PCButton_X>RM_ButtonB</_PCButton_X>
               <rect name=\"NOGLOW_BRANCH\">
                 <rect name=\"RM_ItemsRect\"><locus>&true;</locus>
                   <image name=\"RM_Items_LeftFilterArrow\"><id>0</id><filename>a.dds</filename><target>&true;</target></image>
                   <text name=\"RM_ItemsTitle\"><id>1</id><font>6</font></text>
                   <image name=\"RM_Items_RightFilterArrow\"><id>2</id><filename>a.dds</filename><target>&true;</target></image>
                   {}
                 </rect>
                 <rect name=\"RM_ContainerRect\"><locus>&true;</locus>
                   <text name=\"RM_MadeAtVariable\"><id>4</id><font>3</font></text>
                   <text name=\"RM_SkillRequirement\"><id>5</id><font>3</font></text>
                   {}
                 </rect>
                 <image name=\"RM_ButtonX\"><id>7</id><target>&false;</target><filename>a.dds</filename></image>
                 <image name=\"RM_ButtonB\"><id>8</id><target>&true;</target><filename>a.dds</filename></image>
                 <image name=\"RM_ItemIcon\"><id>9</id><visible>&false;</visible></image>
                 <rect name=\"RM_ItemData\"><id>10</id></rect>
               </rect>
               <template name=\"RM_list_template\"><hotrect name=\"RM_list_template_container\">{LIST_ITEM}<id>15</id>
                 <_Value></_Value>
                 <text name=\"ListItemText\"><font>2</font><string><copy src=\"parent()\" trait=\"string\"/></string></text>
               </hotrect></template>
             </menu>",
            list_box("RM_Items_InventoryList", 3, 462.0, "365"),
            list_box("RM_Items_IngredientList", 6, 442.0, "225"),
        )
    }

    fn line(form: u32, name: &str, sub: u32, makeable: bool) -> RecipeLine {
        RecipeLine {
            form,
            name: name.into(),
            sub,
            makeable,
            skill: if makeable {
                String::new()
            } else {
                "Science 40".into()
            },
            ingredients: vec![
                Ingredient {
                    name: "Scrap".into(),
                    have: 2,
                    need: 2,
                },
                Ingredient {
                    name: "Water".into(),
                    have: 0,
                    need: 1,
                },
            ],
            icon: String::new(),
        }
    }

    fn opened() -> (Ui, RecipeMenu) {
        let mut ui = test_support::ui();
        let mut m = RecipeMenu::new(0);
        m.menu = test_support::load(&mut ui, &recipe_menu(), &mut m);
        let lines = vec![
            line(1, "Bullets", 20, false),
            line(2, "Stew", 10, true),
            line(3, "Tea", 10, true),
        ];
        let filters = vec![
            Filter {
                form: 20,
                name: "Ammo".into(),
            },
            Filter {
                form: 10,
                name: "Aid".into(),
            },
        ];
        assert!(m.open(&mut ui, "Workbench", lines, filters));
        (ui, m)
    }

    fn row(m: &RecipeMenu, value: i32) -> TileId {
        m.lists[0]
            .items
            .iter()
            .find(|i| i.value == value)
            .unwrap()
            .tile
    }

    #[test]
    fn opening_lists_every_recipe_with_nothing_chosen() {
        let (mut ui, m) = opened();
        assert_eq!(m.lists[0].items.len(), 3);
        assert_eq!(m.chosen, None);
        assert_eq!(
            ui.string(m.tiles[1].unwrap(), t::STRING).as_deref(),
            Some("RECIPES")
        );
        // A recipe that can't be made yet looks failed.
        assert!(ui.tiles[row(&m, 0)].failed);
        assert!(!ui.tiles[row(&m, 1)].failed);
        // Accept is off.
        assert_eq!(ui.number(m.tiles[7].unwrap(), t::TARGET), 0.0);
    }

    #[test]
    fn choosing_a_row_shows_its_details_and_turns_accept_on() {
        let (mut ui, mut m) = opened();
        m.lists[0].select(&mut ui, Some(row(&m, 1)));
        m.mouseover(&mut ui, ROW_ID, row(&m, 1));
        assert_eq!(m.chosen, Some(1));
        assert_eq!(
            ui.string(m.tiles[4].unwrap(), t::STRING).as_deref(),
            Some("Workbench")
        );
        assert_eq!(m.lists[1].items.len(), 2);
        let value = RecipeMenu::custom(&mut ui, "_Value");
        let water = m.lists[1].items[1].tile;
        assert_eq!(ui.string(water, value).as_deref(), Some("0/1"));
        // The ingredient the player lacks looks failed.
        assert!(ui.tiles[water].failed);
        assert!(!ui.tiles[m.lists[1].items[0].tile].failed);
        assert_eq!(ui.number(m.tiles[7].unwrap(), t::TARGET), 1.0);
        // The one that can't be made: its skill shows and Accept goes off.
        m.lists[0].select(&mut ui, Some(row(&m, 0)));
        m.mouseover(&mut ui, ROW_ID, row(&m, 0));
        assert_eq!(
            ui.string(m.tiles[5].unwrap(), t::STRING).as_deref(),
            Some("Science 40")
        );
        assert_eq!(ui.number(m.tiles[7].unwrap(), t::TARGET), 0.0);
    }

    #[test]
    fn accept_asks_to_make_only_a_makeable_choice() {
        let (mut ui, mut m) = opened();
        m.click(&mut ui, 7, None, 0.0);
        assert!(m.requests.is_empty());
        m.lists[0].select(&mut ui, Some(row(&m, 0)));
        m.mouseover(&mut ui, ROW_ID, row(&m, 0));
        m.click(&mut ui, 7, None, 0.0);
        assert!(m.requests.is_empty());
        m.lists[0].select(&mut ui, Some(row(&m, 2)));
        m.mouseover(&mut ui, ROW_ID, row(&m, 2));
        m.click(&mut ui, 7, None, 0.0);
        assert_eq!(m.requests, [Request::Make(3)]);
        assert!(!m.closed);
    }

    #[test]
    fn the_arrows_go_through_the_sub_categories() {
        let (mut ui, mut m) = opened();
        m.click(&mut ui, 2, None, 0.0);
        assert_eq!(m.filter, 1);
        assert_eq!(m.lists[0].items.len(), 1);
        assert_eq!(
            ui.string(m.tiles[1].unwrap(), t::STRING).as_deref(),
            Some("AMMO")
        );
        assert!(m.special_key(&mut ui, menu::special::RIGHT, 0.0));
        assert_eq!(m.lists[0].items.len(), 2);
        assert_eq!(
            ui.string(m.tiles[1].unwrap(), t::STRING).as_deref(),
            Some("AID")
        );
        // Round to all, and back from all to the last.
        m.click(&mut ui, 2, None, 0.0);
        assert_eq!((m.filter, m.lists[0].items.len()), (0, 3));
        m.click(&mut ui, 0, None, 0.0);
        assert_eq!(m.filter, 2);
    }

    #[test]
    fn after_making_the_list_follows_what_the_player_has() {
        let (mut ui, mut m) = opened();
        m.lists[0].select(&mut ui, Some(row(&m, 2)));
        m.mouseover(&mut ui, ROW_ID, row(&m, 2));
        let mut lines = m.lines.clone();
        lines[2].makeable = false;
        m.refreshed(&mut ui, lines);
        assert_eq!(m.chosen, Some(2));
        assert_eq!(ui.number(m.tiles[7].unwrap(), t::TARGET), 0.0);
        assert!(ui.tiles[row(&m, 2)].failed);
    }

    #[test]
    fn exit_closes() {
        let (mut ui, mut m) = opened();
        m.click(&mut ui, 8, None, 0.0);
        assert!(m.closed);
    }
}
