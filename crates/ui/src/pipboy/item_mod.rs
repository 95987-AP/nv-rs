//! The Pip-Boy's weapon mod screen (`menus\item_mod_menu.xml`, class
//! `ItemModMenu` 1061, vtable `01073b7c` in FalloutNV.exe): ITEMS' Mod
//! button (id 19, X) on a chosen weapon opens it (`00780140` case 0x13:
//! `UIMenuMode`); a mod from the player's things is fitted to it. The rules
//! are `world::weapon_mods`'. Read from the code:
//!
//! * opening (`00783440`, `00784710`): the weapon (taken off while it's
//!   modded, put back after); its condition on the health card (4, `user0`
//!   condition / 100), its damage on the figure card (5), its name (0); the
//!   list (1, `IMM_ItemModListTemplate`, `007840f0`): the mods fitted to
//!   it, then the player's mods for it that aren't (`004d4ca0`); the fitted
//!   ones dimmed (alpha 127, `007836d0`) and sorted first (`00783810`).
//! * the pointer on a mod (`00783ed0`, id 0xf): its description (`DESC`)
//!   on 13, the button (11) bright.
//! * choosing a mod not fitted (`007838a0` case 0xf → `00783af0`): fitted,
//!   one used up, `UIItemGunsSmallUp`, the list filled again. E, Cancel (12,
//!   case 0xc): `UIMenuMode`, back to ITEMS.
//!
//! Not here: the hold-to-confirm meter the controller fills (`007b5370` to
//! 255), the Pip-Boy's scroll knob.

use super::{by_id, text, trait_id, Action, Key};
use crate::listbox::ListBox;
use crate::names::t;
use crate::tile::{TileId, Ui};

/// The menu's file.
pub const FILE: &str = "menus\\item_mod_menu.xml";
/// The list's template.
pub const TEMPLATE: &str = "IMM_ItemModListTemplate";
/// A line's id (`00783ed0`).
pub const LINE_ID: i32 = 0xf;
/// The Cancel button.
pub const CANCEL: i32 = 12;
/// The sounds: a mod fitted (the exe's string has a trailing space), the
/// menu changing.
pub const FIT_SOUND: &str = "UIItemGunsSmallUp";
pub const MENU_SOUND: &str = "UIMenuMode";
/// A fitted mod's line alpha (`007836d0`: 0x42ff0000 = 127).
pub const FITTED_ALPHA: f32 = 127.0;

/// One mod on the list.
#[derive(Debug, Clone, PartialEq)]
pub struct ModRow {
    pub form: u32,
    pub name: String,
    pub description: String,
    /// Already fitted to the weapon.
    pub fitted: bool,
}

/// What the screen shows.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemModInput {
    pub weapon: u32,
    pub name: String,
    /// Its condition in percent.
    pub condition: f32,
    pub icon: Option<String>,
    /// Its damage as the Pip-Boy shows it.
    pub damage: i32,
    pub rows: Vec<ModRow>,
}

/// The mod screen.
pub struct ItemModMenu {
    pub menu: TileId,
    pub list: ListBox,
    pub rows: Vec<ModRow>,
    pub input: Option<ItemModInput>,
}

impl ItemModMenu {
    /// Reads the menu (hidden).
    pub fn load(
        ui: &mut Ui,
        read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<ItemModMenu, String> {
        let menu = super::load_menu(ui, FILE, read)?;
        let list_tile = by_id(ui, menu, 1).unwrap_or(menu);
        ui.set_number(menu, t::VISIBLE, 0.0);
        Ok(ItemModMenu {
            menu,
            list: ListBox::new(menu, list_tile, TEMPLATE),
            rows: Vec::new(),
            input: None,
        })
    }

    fn tile(&self, ui: &Ui, id: i32) -> Option<TileId> {
        by_id(ui, self.menu, id)
    }

    /// Opens it on a weapon and shows it.
    pub fn open(&mut self, ui: &mut Ui, input: ItemModInput) {
        ui.set_number(self.menu, t::VISIBLE, 1.0);
        if let Some(icon) = self.tile(ui, 3) {
            match &input.icon {
                Some(path) => {
                    ui.set_string(icon, t::FILENAME, path);
                    ui.set_number(icon, t::VISIBLE, 1.0);
                }
                None => ui.set_number(icon, t::VISIBLE, 0.0),
            }
        }
        self.fill(ui, input);
    }

    /// Fills it (`00784710`, again after a fit).
    pub fn fill(&mut self, ui: &mut Ui, input: ItemModInput) {
        if let Some(name) = self.tile(ui, 0) {
            ui.set_string(name, t::STRING, &input.name);
        }
        if let Some(card) = self.tile(ui, 4) {
            ui.set_number(card, t::USER0, input.condition / 100.0);
            ui.set_number(card, t::VISIBLE, 1.0);
        }
        if let Some(card) = self.tile(ui, 5) {
            let (title, value) = (trait_id(ui, "_Title"), trait_id(ui, "_Value"));
            let s = text(ui, "sInventoryDamage");
            ui.set_string(card, title, &s);
            ui.set_number(card, value, input.damage as f32);
        }
        self.list.clear(ui);
        // The fitted ones first (`00783810`), each group in its order.
        let mut rows = input.rows.clone();
        rows.sort_by_key(|r| !r.fitted);
        for row in &rows {
            let Some(tile) = self.list.add(ui, Some(&row.name)) else {
                continue;
            };
            ui.set_number(tile, t::ID, LINE_ID as f32);
            let index = ui.number(tile, t::LISTINDEX);
            ui.set_number(tile, t::VISIBLE, if index >= 0.0 { 1.0 } else { 0.0 });
            if row.fitted {
                ui.set_number(tile, t::ALPHA, FITTED_ALPHA);
            }
        }
        self.rows = rows;
        self.input = Some(input);
        self.list.select(ui, Some(0));
        self.chosen_line(ui);
        ui.refresh();
    }

    /// The line chosen now (`00783ed0`): its description; the button bright.
    fn chosen_line(&mut self, ui: &mut Ui) {
        let row = self.list.selected.and_then(|i| self.rows.get(i)).cloned();
        if let Some(desc) = self.tile(ui, 13) {
            let s = row
                .as_ref()
                .map_or(String::new(), |r| r.description.clone());
            ui.set_string(desc, t::STRING, &s);
        }
        if let (Some(b), Some(_)) = (self.tile(ui, 11), row) {
            let line_alpha = trait_id(ui, "_line_alpha");
            ui.set_number(b, line_alpha, 255.0);
        }
        ui.refresh();
    }

    /// Hides it.
    pub fn hide(&mut self, ui: &mut Ui) {
        ui.set_number(self.menu, t::VISIBLE, 0.0);
        self.input = None;
        ui.refresh();
    }

    /// A key: Up and Down choose a line (`UIPipBoyScroll`), Enter fits the
    /// chosen mod when it isn't fitted yet (`007838a0` case 0xf). Cancel is
    /// [`super::Pipboy::cancel_item_mod`]'s.
    pub fn key(&mut self, ui: &mut Ui, key: Key) -> Vec<Action> {
        let mut out = Vec::new();
        match key {
            Key::Up | Key::Down => {
                let n = self.rows.len();
                if n == 0 {
                    return out;
                }
                let at = self.list.selected.unwrap_or(0);
                let next = if key == Key::Down {
                    (at + 1).min(n - 1)
                } else {
                    at.saturating_sub(1)
                };
                if next != at {
                    self.list.select(ui, Some(next));
                    out.push(Action::Sound("UIPipBoyScroll".into()));
                    self.chosen_line(ui);
                }
            }
            Key::Activate => {
                let row = self.list.selected.and_then(|i| self.rows.get(i));
                if let (Some(row), Some(input)) = (row, self.input.as_ref()) {
                    if !row.fitted {
                        out.push(Action::FitMod {
                            weapon: input.weapon,
                            item: row.form,
                        });
                    }
                }
            }
            _ => {}
        }
        out
    }
}
