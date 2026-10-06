//! The two menus ITEMS opens on the Pip-Boy's screen for the chosen item:
//! Repair (`repair_menu.xml`, the `RepairMenu` class 0x40b, R) and the
//! weapon mod menu (`item_mod_menu.xml`, `ItemModMenu` 0x425, X). Both
//! files' `id` is `&pipboymenu;`: drawn in the Pip-Boy's picture, in
//! ITEMS' place (`007044c0` / `007046f0` show them, `007048f0` turns back
//! to ITEMS).
//!
//! Repair (`007b7020` sets it up for the item, `RepairMenu::InitRepairMenu`
//! (Xbox PDB); `007b6aa0` lists, `007b57f0` fills a row, `007b5950` sorts,
//! `007b6120` mouse-over, `007b5b40` click): the broken item's condition
//! and damage (or DT, DR) on the left cards, the items that can mend it
//! listed with their condition meters, the broken one first between the
//! brackets; the pointer on another shows what mending with it would give
//! (`world::repair::repaired_condition`), a click mends it with that one
//! (`UIRepairWeapon`), then the menu closes when the item is at 99% or
//! nothing else is left to mend with, else it is set up again.
//!
//! Mods (`00784710` sets it up, `007840f0` lists, `007836d0` fills a row,
//! `00783810` sorts, `00783ed0` mouse-over, `007838a0` click): the weapon's
//! name, condition and damage; the mods fitted (dimmed, marked) and the
//! carried mods that fit; the pointer on one shows its description and
//! picture; a click fits it (`UIItemGunsSmallUp `). An equipped weapon is
//! taken off while the menu is open and put back on leaving it.

use super::{by_id, text, trait_id, Action, ItemLine, ItemTab, Key, PipboyInput};
use crate::listbox::ListBox;
use crate::names::{op, t};
use crate::tile::{Operand, TileId, Ui};

pub const REPAIR_FILE: &str = "menus\\repair_menu.xml";
pub const MOD_FILE: &str = "menus\\item_mod_menu.xml";
/// The classes' numbers (`007b54b0`, `007833d0`).
pub const REPAIR_CLASS: i32 = 0x40b;
pub const MOD_CLASS: i32 = 0x425;

/// The rows' ids (`007b57f0`: 0x0e; `007836d0`: 0x0f) and the Cancel /
/// Exit button's (12, E).
pub const REPAIR_ROW_ID: i32 = 0x0e;
pub const MOD_ROW_ID: i32 = 0x0f;
pub const CANCEL_ID: i32 = 12;

/// A row's alpha that can be clicked (`0101e568`: 255), and a fitted
/// mod's dimmed one (`01073c54`: 127.5).
const FULL_ALPHA: f32 = 255.0;
const FITTED_ALPHA: f32 = 127.5;

/// The condition meter's width per point of condition (`010290b8`: 0.6, a
/// 60-wide meter at 100%).
const METER_PER_POINT: f32 = 0.6;

/// "--" (`01047008`).
const NONE_TEXT: &str = "--";

/// `004bd510(v, 1)`.
fn round_half_up(v: f32) -> i32 {
    let whole = v.trunc();
    whole as i32 + i32::from(v - whole >= 0.5)
}

/// A C `printf` of the menus' few formats: `%d` / `%i`, `%.0f`, `%s`.
fn format(pattern: &str, args: &[String]) -> String {
    let mut out = String::new();
    let mut args = args.iter();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut spec = String::new();
        while let Some(&n) = chars.peek() {
            spec.push(n);
            chars.next();
            if n.is_ascii_alphabetic() || n == '%' {
                break;
            }
        }
        if spec == "%" {
            out.push('%');
        } else if let Some(a) = args.next() {
            out.push_str(a);
        }
    }
    out
}

/// A weapon's damage at another condition (0 to 1): the card's damage
/// (`006450f0` at the item's condition) scaled by the worn-weapon
/// multipliers (`00646d00`).
fn damage_at(item: &ItemLine, condition: f32) -> Option<f32> {
    let now = item.condition.unwrap_or(1.0);
    let d = item.damage?;
    Some(
        d * world::npc_combat::condition_mult(condition)
            / world::npc_combat::condition_mult(now).max(1e-6),
    )
}

/// The stat card beside a condition card (`007b7020`): a weapon's damage
/// "DAM"; apparel's DT, else its DR, else "DR" "--"; anything else "DR"
/// "--". (title, value).
fn stat_card(ui: &Ui, item: &ItemLine, condition: f32) -> (String, String, f32) {
    match item.tab {
        ItemTab::Weapons if item.damage.is_some() => {
            let d = damage_at(item, condition).unwrap_or(0.0);
            let v = round_half_up(d);
            (text(ui, "sInventoryDamage"), v.to_string(), v as f32)
        }
        ItemTab::Apparel => {
            let dt = item.damage_threshold.unwrap_or(0.0);
            let dr = item.damage_resistance.unwrap_or(0.0);
            if dt > 0.0 {
                (
                    text(ui, "sInventoryDamageThreshold"),
                    format!("{}", dt as i32),
                    dt,
                )
            } else if dr > 0.0 {
                (
                    text(ui, "sInventoryDamageResistance"),
                    format!("{}", dr as i32),
                    dr,
                )
            } else {
                (
                    text(ui, "sInventoryDamageResistance"),
                    NONE_TEXT.into(),
                    0.0,
                )
            }
        }
        _ => (
            text(ui, "sInventoryDamageResistance"),
            NONE_TEXT.into(),
            0.0,
        ),
    }
}

/// A row of the Repair menu.
#[derive(Debug, Clone, PartialEq)]
pub struct RepairRow {
    pub form: u32,
    pub text: String,
    /// Condition, percent.
    pub condition: f32,
    pub equipped: bool,
    /// The item being repaired (`_IsRepairItem`).
    pub broken: bool,
    /// What mending with it gives, 0 to 1 (the row's `user0`).
    pub repaired: f32,
}

/// The rows' order (`007b5950`): the broken item first; rows that can't be
/// clicked (alpha below 255) last; higher condition first (the other way
/// among rows that can't be clicked); then equipped first.
// Translated from 007b5950 (decompiled, FalloutNV.exe 1.4.0.525)
fn repair_order(a: &RepairRow, b: &RepairRow) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    match (a.broken, b.broken) {
        (true, false) => return Less,
        (false, true) => return Greater,
        _ => {}
    }
    let mut c = if b.condition > a.condition {
        Greater
    } else if b.condition < a.condition {
        Less
    } else {
        Equal
    };
    if c == Equal {
        c = match (a.equipped, b.equipped) {
            (true, _) => Less,
            (false, true) => Greater,
            _ => Equal,
        };
    }
    c
}

/// The Repair menu.
pub struct RepairMenu {
    pub menu: TileId,
    pub list: ListBox,
    /// The item being repaired (`011da760`).
    pub broken: Option<u32>,
    pub rows: Vec<RepairRow>,
    /// Set up again on the next fill (after a repair).
    dirty: bool,
    /// The last row the pointer was over (`011da7d8`, its `listindex`).
    pub(crate) hovered: Option<usize>,
    /// The broken item's condition card's value and its stat (for the
    /// improvement texts).
    broken_share: f32,
    broken_stat: f32,
}

impl RepairMenu {
    pub fn load(
        ui: &mut Ui,
        read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<RepairMenu, String> {
        let menu = super::load_menu(ui, REPAIR_FILE, read)?;
        ui.set_number(menu, t::VISIBLE, 0.0);
        let list_tile = by_id(ui, menu, 1).unwrap_or(menu);
        Ok(RepairMenu {
            menu,
            list: ListBox::new(menu, list_tile, "RM_RepairListTemplate"),
            broken: None,
            rows: Vec::new(),
            dirty: false,
            hovered: None,
            broken_share: 0.0,
            broken_stat: 0.0,
        })
    }

    fn item<'a>(&self, input: &'a PipboyInput, form: u32) -> Option<&'a ItemLine> {
        input.items.iter().find(|i| i.form == form)
    }

    /// Sets the menu up for an item (`007b7020` with the item): its cards,
    /// picture, the list, the skill; the first row moused over.
    pub fn open(&mut self, ui: &mut Ui, broken: u32, input: &PipboyInput) -> Vec<Action> {
        self.broken = Some(broken);
        self.hovered = None;
        self.build(ui, input)
    }

    /// Set up again with the same item (`007b7020(0)`).
    fn build(&mut self, ui: &mut Ui, input: &PipboyInput) -> Vec<Action> {
        self.dirty = false;
        let Some(item) = self.broken.and_then(|f| self.item(input, f)).cloned() else {
            return Vec::new();
        };
        let title = trait_id(ui, "_Title");
        let value = trait_id(ui, "_Value");
        let condition = item.condition.unwrap_or(1.0) * 100.0;
        let share = condition / 100.0;
        if let Some(card) = by_id(ui, self.menu, 4) {
            ui.set_number(card, t::USER0, share);
            ui.set_number(card, t::VISIBLE, 1.0);
        }
        let (stat_title, stat_value, stat) = stat_card(ui, &item, share);
        if let Some(card) = by_id(ui, self.menu, 5) {
            ui.set_string(card, title, &stat_title);
            ui.set_string(card, value, &stat_value);
        }
        self.broken_share = share;
        self.broken_stat = stat;
        if let Some(icon) = by_id(ui, self.menu, 3) {
            ui.set_string(icon, t::FILENAME, item.icon.as_deref().unwrap_or(""));
        }
        self.fill_list(ui, &item, input);
        // The player's Repair skill on the headline card.
        if let Some(card) = by_id(ui, self.menu, 0) {
            ui.set_string(card, value, &input.repair_skill.to_string());
        }
        // The first row moused over (`+0x10` with the list's first).
        let mut out = Vec::new();
        if let Some(&row) = self.list.rows.first() {
            out.extend(self.mouseover(ui, REPAIR_ROW_ID, row, input));
        }
        ui.refresh();
        out
    }

    /// The list (`007b6aa0`): the carried items that can mend the broken
    /// one (`004d4bd0`: itself and its repair list); the broken one marked;
    /// another of the same item (the world keeps one condition for both)
    /// unless that one is equipped; each row filled (`007b57f0`) and the
    /// rows sorted (`007b5950`). The brackets frame the broken one's row
    /// (its `y`, its height less 1).
    fn fill_list(&mut self, ui: &mut Ui, item: &ItemLine, input: &PipboyInput) {
        let s = &input.repair;
        let skill = input.repair_skill;
        let broken_condition = item.condition.unwrap_or(1.0) * 100.0;
        let mut rows = Vec::new();
        for &form in &item.menders {
            let Some(other) = input.items.iter().find(|i| i.form == form) else {
                continue;
            };
            let condition = other.condition.unwrap_or(1.0) * 100.0;
            let row = |broken: bool, count: i32| RepairRow {
                form,
                text: super::items::row_text(&ItemLine {
                    count,
                    ..other.clone()
                }),
                condition,
                equipped: other.equipped && (broken || form != item.form),
                broken,
                repaired: (world::repair::repaired_condition(
                    s,
                    skill,
                    condition,
                    broken_condition,
                )
                .0 / 100.0)
                    .min(1.0),
            };
            if form == item.form {
                rows.push(row(true, 1));
                if other.count >= 2 {
                    rows.push(row(false, other.count - 1));
                }
            } else {
                rows.push(row(false, other.count));
            }
        }
        rows.sort_by(repair_order);
        self.list.clear(ui);
        let is_repair = trait_id(ui, "_IsRepairItem");
        for r in &rows {
            let Some(tile) = self.list.add(ui, Some(&r.text)) else {
                continue;
            };
            ui.set_number(tile, t::ID, REPAIR_ROW_ID as f32);
            ui.set_number(tile, is_repair, if r.broken { 1.0 } else { 0.0 });
            if let Some(meter) = by_id(ui, tile, 15) {
                ui.set_number(meter, t::WIDTH, r.condition * METER_PER_POINT);
            }
            if let Some(marker) = ui.find_below(tile, "RM_Template_ItemMarker") {
                ui.set_number(marker, t::VISIBLE, if r.equipped { 1.0 } else { 0.0 });
            }
            ui.set_number(tile, t::USER0, r.repaired);
            if r.broken {
                if let Some(bracket) = by_id(ui, self.menu, 2) {
                    ui.set_number(bracket, t::HEIGHT, 0.0);
                    ui.add_action(
                        bracket,
                        t::HEIGHT,
                        op::COPY,
                        Operand::Link {
                            tile,
                            trait_id: t::HEIGHT,
                        },
                    );
                    ui.add_action(bracket, t::HEIGHT, op::SUB, Operand::Constant(1.0));
                    ui.set_number(bracket, t::Y, 0.0);
                    ui.add_action(
                        bracket,
                        t::Y,
                        op::COPY,
                        Operand::Link {
                            tile,
                            trait_id: t::Y,
                        },
                    );
                }
            }
        }
        self.rows = rows;
        self.list.choose(ui, None);
    }

    /// Every frame: set up again after a repair.
    pub fn fill(&mut self, ui: &mut Ui, input: &PipboyInput) -> Vec<Action> {
        if self.dirty {
            return self.build(ui, input);
        }
        Vec::new()
    }

    /// The pointer onto a row (`007b6120`): the knob when the row changes;
    /// on the broken one with others listed "CHOOSE ITEM TO REPAIR WITH";
    /// on another the condition and stat it would give and their gains, or
    /// what stops it (the row's `user1` above 100: "CANNOT REPAIR PAST",
    /// above 0: the skill needed; no code sets it, so the file's 0 holds).
    // Translated from 007b6120 (decompiled, FalloutNV.exe 1.4.0.525)
    pub fn mouseover(
        &mut self,
        ui: &mut Ui,
        id: i32,
        tile: TileId,
        input: &PipboyInput,
    ) -> Vec<Action> {
        let mut out = Vec::new();
        if id != REPAIR_ROW_ID {
            return out;
        }
        let Some(index) = self.list.index_of(tile) else {
            return out;
        };
        if self.hovered != Some(index) {
            self.hovered = Some(index);
            out.push(Action::Sound("UIPipBoyScroll".into()));
        }
        self.list.choose(ui, Some(index));
        let Some(item) = self.broken.and_then(|f| self.item(input, f)).cloned() else {
            return out;
        };
        let Some(row) = self.rows.get(index).cloned() else {
            return out;
        };
        // The (pad's) Repair button: "Maintain" at 75% and above (50% for
        // apparel), else "Repair".
        let at = if item.tab == ItemTab::Apparel {
            50.0
        } else {
            75.0
        };
        if let Some(button) = by_id(ui, self.menu, 11) {
            let name = if item.condition.unwrap_or(1.0) * 100.0 >= at {
                "sMaintainItem"
            } else {
                "sRepairItem"
            };
            let v = text(ui, name);
            ui.set_string(button, t::STRING, &v);
        }
        let line_alpha = trait_id(ui, "_line_alpha");
        let choose = by_id(ui, self.menu, 8);
        let fixed = by_id(ui, self.menu, 9);
        let show_choose = |ui: &mut Ui, s: &str| {
            if let Some(c) = choose {
                ui.set_string(c, t::STRING, s);
                ui.set_number(c, t::VISIBLE, 1.0);
            }
            if let Some(f) = fixed {
                ui.set_number(f, t::VISIBLE, 0.0);
            }
        };
        if row.broken {
            if self.rows.len() > 1 {
                let s = text(ui, "sSelectItemToRepair");
                show_choose(ui, &s);
            }
            if let Some(button) = by_id(ui, self.menu, 11) {
                ui.set_number(button, line_alpha, 128.0);
            }
            return out;
        }
        let needed = ui.number(tile, t::USER0 + 1) as i32;
        if needed > 100 {
            let s = format(
                &text(ui, "sCantRepairPastMax"),
                &[format!("{:.0}", input.repair.skill_max * 10.0), "%".into()],
            );
            show_choose(ui, &s);
            return out;
        }
        if needed > 0 {
            let s = format(
                &text(ui, "sRepairSkillTooLow"),
                &[needed.to_string(), "%".into()],
            );
            show_choose(ui, &s);
            return out;
        }
        if let Some(button) = by_id(ui, self.menu, 11) {
            ui.set_number(button, line_alpha, 255.0);
        }
        if let Some(f) = fixed {
            ui.set_number(f, t::USER0, row.repaired);
            ui.set_number(f, t::VISIBLE, 1.0);
        }
        if let Some(c) = choose {
            ui.set_number(c, t::VISIBLE, 0.0);
        }
        let title = trait_id(ui, "_Title");
        let value = trait_id(ui, "_Value");
        let (stat_title, stat_value, stat) = stat_card(ui, &item, row.repaired);
        if let Some(card) = by_id(ui, self.menu, 10) {
            ui.set_string(card, title, &stat_title);
            ui.set_string(card, value, &stat_value);
        }
        let gain = ((row.repaired - self.broken_share) * 100.0).max(0.0);
        let stat_gain = (stat - self.broken_stat).max(0.0);
        if let Some(t6) = by_id(ui, self.menu, 6) {
            let s = format!("+{}%", (gain + 0.5) as i32);
            ui.set_string(t6, t::STRING, &s);
        }
        if let Some(t7) = by_id(ui, self.menu, 7) {
            let s = format!("+{}", (stat_gain + 0.5) as i32);
            ui.set_string(t7, t::STRING, &s);
        }
        out
    }

    /// The pointer off a row (`007b6a00`): the repaired card hides, and the
    /// "choose" text with more than one row.
    pub fn unmouseover(&mut self, ui: &mut Ui, id: i32) {
        if id != REPAIR_ROW_ID {
            return;
        }
        if let Some(f) = by_id(ui, self.menu, 9) {
            ui.set_number(f, t::VISIBLE, 0.0);
        }
        if self.rows.len() > 1 {
            if let Some(c) = by_id(ui, self.menu, 8) {
                ui.set_number(c, t::VISIBLE, 0.0);
            }
        }
    }

    /// A click (`007b5b40`): Cancel (12) goes back to ITEMS
    /// (`UIMenuMode`); a row other than the broken item mends it with that
    /// one (`UIRepairWeapon`), then closes when it comes to 99% or only the
    /// broken one is left, else sets up again. Returns whether it closes.
    // Translated from 007b5b40 (decompiled, FalloutNV.exe 1.4.0.525)
    pub fn click(&mut self, ui: &mut Ui, id: i32, tile: Option<TileId>) -> (Vec<Action>, bool) {
        let mut out = Vec::new();
        match id {
            CANCEL_ID => {
                out.push(Action::Sound("UIMenuMode".into()));
                return (out, true);
            }
            REPAIR_ROW_ID => {
                let Some(index) = tile
                    .and_then(|t| self.list.index_of(t))
                    .or(self.list.selected)
                else {
                    return (out, false);
                };
                let Some(row) = self.rows.get(index).cloned() else {
                    return (out, false);
                };
                let row_tile = self.list.rows[index];
                // The row's text's alpha (`007b5370`: the code dims a row
                // by its `ListItemText`).
                let alpha = ui
                    .find_below(row_tile, "ListItemText")
                    .map_or(FULL_ALPHA, |t| ui.number(t, t::ALPHA));
                if alpha < FULL_ALPHA || row.broken || ui.number(row_tile, t::VISIBLE) == 0.0 {
                    return (out, false);
                }
                let Some(broken) = self.broken else {
                    return (out, false);
                };
                out.push(Action::Repair {
                    broken,
                    with: row.form,
                });
                out.push(Action::Sound("UIRepairWeapon".into()));
                // The row used goes from the list; with only the broken one
                // left, or the item at 99% and above, the menu closes.
                let left = self.rows.len() - 1;
                if left == 1 || row.repaired * 100.0 >= 99.0 {
                    out.push(Action::Sound("UIMenuMode".into()));
                    return (out, true);
                }
                self.dirty = true;
            }
            _ => {}
        }
        (out, false)
    }

    /// Keys: up and down choose a row (as the pointer does), the A button
    /// clicks it.
    pub fn key(&mut self, ui: &mut Ui, key: Key, input: &PipboyInput) -> (Vec<Action>, bool) {
        match key {
            Key::Up | Key::Down => {
                self.list.step(ui, if key == Key::Down { 1 } else { -1 });
                let row = self.list.selected.map(|i| self.list.rows[i]);
                let out = row.map_or(Vec::new(), |r| self.mouseover(ui, REPAIR_ROW_ID, r, input));
                (out, false)
            }
            Key::Activate => {
                let row = self.list.selected.map(|i| self.list.rows[i]);
                self.click(ui, REPAIR_ROW_ID, row)
            }
            _ => (Vec::new(), false),
        }
    }
}

/// A row of the mod menu: a mod fitted or carried.
#[derive(Debug, Clone, PartialEq)]
pub struct ModRow {
    pub form: u32,
    pub text: String,
    pub fitted: bool,
    pub description: String,
    pub icon: Option<String>,
}

/// The weapon mod menu.
pub struct ModMenu {
    pub menu: TileId,
    pub list: ListBox,
    /// The weapon (`011d9f58`).
    pub weapon: Option<u32>,
    pub rows: Vec<ModRow>,
    /// The weapon was equipped and taken off on opening (`011d9f40`).
    pub unequipped: bool,
    dirty: bool,
    pub(crate) hovered: Option<usize>,
}

impl ModMenu {
    pub fn load(
        ui: &mut Ui,
        read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<ModMenu, String> {
        let menu = super::load_menu(ui, MOD_FILE, read)?;
        ui.set_number(menu, t::VISIBLE, 0.0);
        let list_tile = by_id(ui, menu, 1).unwrap_or(menu);
        Ok(ModMenu {
            menu,
            list: ListBox::new(menu, list_tile, "IMM_ItemModListTemplate"),
            weapon: None,
            rows: Vec::new(),
            unequipped: false,
            dirty: false,
            hovered: None,
        })
    }

    /// Sets the menu up for a weapon (`00784710(weapon, 1)`): an equipped
    /// weapon is taken off first (and put back on leaving).
    pub fn open(&mut self, ui: &mut Ui, weapon: u32, input: &PipboyInput) -> Vec<Action> {
        self.weapon = Some(weapon);
        self.unequipped = false;
        self.hovered = None;
        let mut out = Vec::new();
        if input.items.iter().any(|i| i.form == weapon && i.equipped) {
            out.push(Action::Equip(weapon));
            self.unequipped = true;
        }
        self.build(ui, input);
        out
    }

    /// The cards, the name, the picture and the list (`00784710`,
    /// `007840f0`): the fitted mods first (dimmed to 127.5, marked), then
    /// the carried mods that fit and aren't fitted, sorted by the marker
    /// (`00783810`).
    fn build(&mut self, ui: &mut Ui, input: &PipboyInput) {
        self.dirty = false;
        let Some(item) = self
            .weapon
            .and_then(|f| input.items.iter().find(|i| i.form == f))
            .cloned()
        else {
            return;
        };
        let title = trait_id(ui, "_Title");
        let value = trait_id(ui, "_Value");
        let share = item.condition.unwrap_or(1.0);
        if let Some(card) = by_id(ui, self.menu, 4) {
            ui.set_number(card, t::USER0, share);
            ui.set_number(card, t::VISIBLE, 1.0);
        }
        let (stat_title, stat_value, _) = stat_card(ui, &item, share);
        if let Some(card) = by_id(ui, self.menu, 5) {
            ui.set_string(card, title, &stat_title);
            ui.set_string(card, value, &stat_value);
        }
        if let Some(name) = by_id(ui, self.menu, 0) {
            ui.set_string(name, t::STRING, &item.name);
        }
        if let Some(icon) = by_id(ui, self.menu, 3) {
            ui.set_string(icon, t::FILENAME, item.icon.as_deref().unwrap_or(""));
        }
        let mut rows = Vec::new();
        for slot in item.mods.iter().filter(|s| s.fitted) {
            rows.push(ModRow {
                form: slot.item,
                text: slot.name.clone(),
                fitted: true,
                description: slot.description.clone(),
                icon: slot.icon.clone(),
            });
        }
        for slot in item.mods.iter().filter(|s| !s.fitted) {
            if item.mods.iter().any(|o| o.fitted && o.item == slot.item) {
                continue;
            }
            if let Some(carried) = input.items.iter().find(|i| i.form == slot.item) {
                rows.push(ModRow {
                    form: slot.item,
                    text: super::items::row_text(carried),
                    fitted: false,
                    description: slot.description.clone(),
                    icon: carried.icon.clone().or_else(|| slot.icon.clone()),
                });
            }
        }
        // Stable: fitted (marker shown) first.
        rows.sort_by_key(|r| !r.fitted);
        self.list.clear(ui);
        for r in &rows {
            let Some(tile) = self.list.add(ui, Some(&r.text)) else {
                continue;
            };
            ui.set_number(tile, t::ID, MOD_ROW_ID as f32);
            let is_mod = trait_id(ui, "_IsItemModItem");
            ui.set_number(tile, is_mod, 1.0);
            if let Some(marker) = ui.find_below(tile, "IMM_Template_ItemMarker") {
                ui.set_number(marker, t::VISIBLE, if r.fitted { 1.0 } else { 0.0 });
            }
            if r.fitted {
                if let Some(text_tile) = ui.find_below(tile, "ListItemText") {
                    ui.set_number(text_tile, t::ALPHA, FITTED_ALPHA);
                }
            }
        }
        self.rows = rows;
        self.list.choose(ui, None);
        if let Some(desc) = by_id(ui, self.menu, 13) {
            ui.set_string(desc, t::STRING, "");
        }
        ui.refresh();
    }

    pub fn fill(&mut self, ui: &mut Ui, input: &PipboyInput) {
        if self.dirty {
            self.build(ui, input);
        }
    }

    /// The pointer onto a row (`00783ed0`): the knob when the row changes,
    /// the mod's description and picture.
    // Translated from 00783ed0 (decompiled, FalloutNV.exe 1.4.0.525)
    pub fn mouseover(&mut self, ui: &mut Ui, id: i32, tile: TileId) -> Vec<Action> {
        let mut out = Vec::new();
        if id != MOD_ROW_ID {
            return out;
        }
        let Some(index) = self.list.index_of(tile) else {
            return out;
        };
        if self.hovered != Some(index) {
            self.hovered = Some(index);
            out.push(Action::Sound("UIPipBoyScroll".into()));
        }
        self.list.choose(ui, Some(index));
        let Some(row) = self.rows.get(index).cloned() else {
            return out;
        };
        if let Some(desc) = by_id(ui, self.menu, 13) {
            ui.set_string(desc, t::STRING, &row.description);
        }
        if let Some(icon) = by_id(ui, self.menu, 3) {
            ui.set_string(icon, t::FILENAME, row.icon.as_deref().unwrap_or(""));
        }
        if let Some(button) = by_id(ui, self.menu, 11) {
            let line_alpha = trait_id(ui, "_line_alpha");
            ui.set_number(button, line_alpha, 255.0);
        }
        out
    }

    /// A click (`007838a0`): Exit (12) puts an equipped weapon back on and
    /// goes back to ITEMS (`UIMenuMode`); a carried mod's row (alpha 255)
    /// fits it (`UIItemGunsSmallUp `, the exe's name with its space) and
    /// sets the menu up again.
    // Translated from 007838a0 (decompiled, FalloutNV.exe 1.4.0.525)
    pub fn click(&mut self, ui: &mut Ui, id: i32, tile: Option<TileId>) -> (Vec<Action>, bool) {
        let mut out = Vec::new();
        match id {
            CANCEL_ID => {
                out.push(Action::Sound("UIMenuMode".into()));
                if std::mem::take(&mut self.unequipped) {
                    if let Some(w) = self.weapon {
                        out.push(Action::Equip(w));
                    }
                }
                return (out, true);
            }
            MOD_ROW_ID => {
                let Some(index) = tile
                    .and_then(|t| self.list.index_of(t))
                    .or(self.list.selected)
                else {
                    return (out, false);
                };
                let Some(row) = self.rows.get(index).cloned() else {
                    return (out, false);
                };
                let alpha = ui
                    .find_below(self.list.rows[index], "ListItemText")
                    .map_or(FULL_ALPHA, |t| ui.number(t, t::ALPHA));
                if alpha < FULL_ALPHA || row.fitted || Some(row.form) == self.weapon {
                    return (out, false);
                }
                let Some(weapon) = self.weapon else {
                    return (out, false);
                };
                out.push(Action::FitMod {
                    weapon,
                    item: row.form,
                });
                out.push(Action::Sound("UIItemGunsSmallUp ".into()));
                self.dirty = true;
            }
            _ => {}
        }
        (out, false)
    }

    pub fn key(&mut self, ui: &mut Ui, key: Key) -> (Vec<Action>, bool) {
        match key {
            Key::Up | Key::Down => {
                self.list.step(ui, if key == Key::Down { 1 } else { -1 });
                let row = self.list.selected.map(|i| self.list.rows[i]);
                let out = row.map_or(Vec::new(), |r| self.mouseover(ui, MOD_ROW_ID, r));
                (out, false)
            }
            Key::Activate => {
                let row = self.list.selected.map(|i| self.list.rows[i]);
                self.click(ui, MOD_ROW_ID, row)
            }
            _ => (Vec::new(), false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `repair_menu.xml` cut down to the tiles the code finds by `id`.
    const REPAIR: &str = r#"<menu name="RepairMenu"><locus>&true;</locus>
      <rect name="skill"><id>0</id></rect>
      <hotrect name="list"><id>1</id><x>0</x><y>100</y><width>400</width><height>400</height><locus>&true;</locus>
        <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
        <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
        <rect name="RM_LeftBracket"><id>2</id></rect>
      </hotrect>
      <image name="icon"><id>3</id></image>
      <rect name="broken"><id>4</id></rect><rect name="bstat"><id>5</id></rect>
      <text name="hgain"><id>6</id></text><text name="sgain"><id>7</id></text>
      <text name="choose"><id>8</id><visible>&false;</visible></text>
      <rect name="fixed"><id>9</id><visible>&false;</visible></rect><rect name="fstat"><id>10</id></rect>
      <image name="RM_RepairButton"><id>11</id></image>
      <image name="RM_CancelButton"><id>12</id></image>
      <template name="RM_RepairListTemplate"><hotrect name="row"><height>30</height><width>300</width><target>&true;</target>
        <y><copy src="me()" trait="_y"/></y>
        <text name="ListItemText"><alpha>255</alpha></text>
        <image name="RM_Template_Meter"><id>15</id></image>
        <image name="RM_Template_ItemMarker"><visible>&false;</visible></image></hotrect></template>
    </menu>"#;

    const MODS: &str = r#"<menu name="ItemModMenu"><locus>&true;</locus>
      <text name="name"><id>0</id></text>
      <hotrect name="list"><id>1</id><x>0</x><y>100</y><width>400</width><height>400</height><locus>&true;</locus>
        <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
        <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
      </hotrect>
      <image name="icon"><id>3</id></image>
      <rect name="health"><id>4</id></rect><rect name="stat"><id>5</id></rect>
      <text name="desc"><id>13</id></text>
      <image name="IMM_CancelButton"><id>12</id></image>
      <template name="IMM_ItemModListTemplate"><hotrect name="row"><height>30</height><width>300</width><target>&true;</target>
        <y><copy src="me()" trait="_y"/></y>
        <text name="ListItemText"><alpha>255</alpha></text>
        <image name="IMM_Template_ItemMarker"><visible>&false;</visible></image></hotrect></template>
    </menu>"#;

    fn pistol(count: i32) -> ItemLine {
        let mut p = crate::pipboy::items::tests::item("9mm Pistol", ItemTab::Weapons);
        p.form = 0xE3778;
        p.count = count;
        p.condition = Some(0.4);
        p.damage = Some(10.0);
        p.menders = vec![0xE3778];
        p.repairable = true;
        p
    }

    /// `007b7020`, `007b6120`, `007b5b40`: two damaged pistols (the world
    /// keeps one condition for both); the broken one first between the
    /// brackets with "choose" shown; the pointer on the other shows the
    /// exe's repair (skill 15, 40% with 40%: 49.25%, +9%), a click mends
    /// with it and, the broken one being all that's left, closes.
    #[test]
    fn repairing_a_pistol_with_another() {
        let mut ui = crate::pipboy::tests::ui();
        let mut read = |p: &str| (p == REPAIR_FILE).then(|| REPAIR.as_bytes().to_vec());
        let mut m = RepairMenu::load(&mut ui, &mut read).unwrap();
        let input = PipboyInput {
            items: vec![pistol(2)],
            repair_skill: 15,
            ..PipboyInput::default()
        };
        m.open(&mut ui, 0xE3778, &input);
        ui.refresh();
        assert_eq!(m.rows.len(), 2);
        assert!(m.rows[0].broken && !m.rows[1].broken);
        let choose = by_id(&ui, m.menu, 8).unwrap();
        assert_eq!(ui.number(choose, t::VISIBLE), 1.0);
        let bracket = by_id(&ui, m.menu, 2).unwrap();
        assert_eq!(ui.number(bracket, t::HEIGHT), 29.0);
        let skill = by_id(&ui, m.menu, 0).unwrap();
        let value = ui.names.lookup("_Value").unwrap();
        assert_eq!(ui.string(skill, value).unwrap(), "15");
        // The meter: 40% × 0.6.
        let meter = by_id(&ui, m.list.rows[1], 15).unwrap();
        assert!((ui.number(meter, t::WIDTH) - 24.0).abs() < 1e-4);
        let row = m.list.rows[1];
        let out = m.mouseover(&mut ui, REPAIR_ROW_ID, row, &input);
        assert_eq!(out, [Action::Sound("UIPipBoyScroll".into())]);
        let fixed = by_id(&ui, m.menu, 9).unwrap();
        assert_eq!(ui.number(fixed, t::VISIBLE), 1.0);
        assert!((ui.number(fixed, t::USER0) - 0.4925).abs() < 1e-4);
        let gain = by_id(&ui, m.menu, 6).unwrap();
        assert_eq!(ui.string(gain, t::STRING).unwrap(), "+9%");
        let (out, close) = m.click(&mut ui, REPAIR_ROW_ID, Some(row));
        assert!(out.contains(&Action::Repair {
            broken: 0xE3778,
            with: 0xE3778
        }));
        assert!(out.contains(&Action::Sound("UIRepairWeapon".into())));
        assert!(close);
        // The broken one's own row does nothing.
        let (out, close) = m.click(&mut ui, REPAIR_ROW_ID, Some(m.list.rows[0]));
        assert!(out.is_empty() && !close);
    }

    /// `00784710`, `007840f0`, `007838a0`: an equipped weapon is taken off;
    /// the carried mod that fits is listed, its description on the
    /// pointer, a click fits it; a fitted mod is listed dimmed and can't
    /// be; Exit puts the weapon back on.
    #[test]
    fn fitting_a_mod() {
        let mut ui = crate::pipboy::tests::ui();
        let mut read = |p: &str| (p == MOD_FILE).then(|| MODS.as_bytes().to_vec());
        let mut m = ModMenu::load(&mut ui, &mut read).unwrap();
        let mut gun = pistol(1);
        gun.equipped = true;
        gun.mods = vec![
            crate::pipboy::ModSlotLine {
                item: 0xEED3D,
                name: "Extended Mags".into(),
                description: "Increases ammunition capacity (+7).".into(),
                icon: None,
                fitted: false,
            },
            crate::pipboy::ModSlotLine {
                item: 0xEED3C,
                name: "Scope".into(),
                description: String::new(),
                icon: None,
                fitted: true,
            },
        ];
        let mut mags = crate::pipboy::items::tests::item("Extended Mags", ItemTab::Misc);
        mags.form = 0xEED3D;
        let input = PipboyInput {
            items: vec![gun, mags],
            ..PipboyInput::default()
        };
        let out = m.open(&mut ui, 0xE3778, &input);
        assert_eq!(out, [Action::Equip(0xE3778)]);
        ui.refresh();
        assert_eq!(
            m.rows
                .iter()
                .map(|r| (r.form, r.fitted))
                .collect::<Vec<_>>(),
            [(0xEED3C, true), (0xEED3D, false)]
        );
        let fitted_text = ui.find_below(m.list.rows[0], "ListItemText").unwrap();
        assert_eq!(ui.number(fitted_text, t::ALPHA), 127.5);
        let (out, _) = m.click(&mut ui, MOD_ROW_ID, Some(m.list.rows[0]));
        assert!(out.is_empty());
        m.mouseover(&mut ui, MOD_ROW_ID, m.list.rows[1]);
        let desc = by_id(&ui, m.menu, 13).unwrap();
        assert_eq!(
            ui.string(desc, t::STRING).unwrap(),
            "Increases ammunition capacity (+7)."
        );
        let (out, close) = m.click(&mut ui, MOD_ROW_ID, Some(m.list.rows[1]));
        assert_eq!(
            out,
            [
                Action::FitMod {
                    weapon: 0xE3778,
                    item: 0xEED3D
                },
                Action::Sound("UIItemGunsSmallUp ".into())
            ]
        );
        assert!(!close);
        let (out, close) = m.click(&mut ui, CANCEL_ID, None);
        assert!(close);
        assert!(out.contains(&Action::Equip(0xE3778)));
    }

    #[test]
    fn the_menus_formats() {
        assert_eq!(
            format("CANNOT REPAIR PAST %.0f%s", &["90".into(), "%".into()]),
            "CANNOT REPAIR PAST 90%"
        );
        assert_eq!(
            format("%d%s REPAIR SKILL NEEDED", &["45".into(), "%".into()]),
            "45% REPAIR SKILL NEEDED"
        );
    }

    #[test]
    fn rows_sort_broken_first_then_by_condition() {
        let row = |form: u32, condition: f32, broken: bool, equipped: bool| RepairRow {
            form,
            text: String::new(),
            condition,
            equipped,
            broken,
            repaired: 0.0,
        };
        let mut rows = [
            row(1, 40.0, false, false),
            row(2, 90.0, false, false),
            row(3, 30.0, true, true),
            row(4, 40.0, false, true),
        ];
        rows.sort_by(repair_order);
        assert_eq!(
            rows.iter().map(|r| r.form).collect::<Vec<_>>(),
            [3, 2, 4, 1]
        );
    }
}
