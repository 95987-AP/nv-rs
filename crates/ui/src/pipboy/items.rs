//! ITEMS (`inventory_menu.xml`, the `InventoryMenu` class): five tabs
//! along a tab line (Weapons, Apparel, Aid, Misc, Ammo), the items of the
//! tab in a list, and a card for the chosen item. Filled as `0077fc10`
//! (setup), `00782a90` (the headline), `00782850` (a row) and `00707e30`
//! (the item card, shared with the container and barter menus) fill it.

use super::{by_id, text, trait_id, Action, ItemLine, ItemTab, Key, PipboyInput};
use crate::listbox::ListBox;
use crate::names::t;
use crate::tabline;
use crate::tile::{TileId, Ui};

/// The item card's cards (`item_stats_display.xml`'s children, in order),
/// and the bit of `00707e30`'s mask that shows each (read from its
/// disassembly: bit 0x1 the first child, 0x2 the second, ... 0x800 the
/// twelfth, 0x1000 the thirteenth).
pub const CARDS: [(&str, u32); 13] = [
    ("DamageResistInfo", 0x1),
    ("DPSInfo", 0x2),
    ("WeightInfo", 0x4),
    ("ValueInfo", 0x8),
    ("ConditionInfo", 0x10),
    ("AmmoInfo", 0x20),
    ("EffectsInfo", 0x40),
    ("ModInfoOne", 0x80),
    ("ModInfoTwo", 0x100),
    ("ModInfoThree", 0x200),
    ("StrengthReqInfo", 0x400),
    ("DAMInfo", 0x800),
    ("DamageThresholdInfo", 0x1000),
];

/// The ITEMS menu.
pub struct ItemsMenu {
    pub menu: TileId,
    pub tab: usize,
    pub list: ListBox,
    pub tabline: Option<TileId>,
    pub tabs: Vec<TileId>,
    card: Option<TileId>,
    icon: Option<TileId>,
    /// The forms of the rows now, in order (0 for the keyring's row).
    pub shown: Vec<u32>,
    filled: Option<(usize, bool, Vec<ItemLine>, bool)>,
    /// The last row the pointer was over (`011d9f34`, its
    /// `listindex`): the knob clicks when it changes.
    hovered: Option<usize>,
    /// The keyring is open (the menu's `_KeyringOpen`, `011d9eb8`): the
    /// list shows the keys (`00782810`).
    pub keyring: bool,
}

/// The keyring's row's `id` (`00782a90`: a row without an item, `sKeyring`,
/// last on the Misc tab when keys are carried).
pub const KEYRING_ID: i32 = 0x1e;
/// The Cancel button's `id` (`IM_CancelButton`, E: shown while the
/// keyring is open; closes it, `00780140` case 10).
pub const CANCEL_ID: i32 = 10;
/// The keyring's picture (`00780ff0` on its row).
pub const KEYRING_PICTURE: &str = "Interface\\Icons\\PipboyImages\\Items\\item_keyring.dds";

/// The rows' `id` (0x1d), which the click and mouse-over handlers look for
/// (`00780140`, `00780ff0`).
pub const ROW_ID: i32 = 0x1d;
/// The Drop button's `id` (`IM_DropButton`, shown with a pad only; the
/// right mouse button clicks it, `00781ba0`).
pub const DROP_ID: i32 = 7;
/// The first tab button's `id` (0x18 Weapons .. 0x1c Ammo, `0077fc10`).
pub const FIRST_TAB_ID: i32 = 0x18;

/// An item's row text (`00782850`): "name (count)" for more than one
/// ("name+ (count)" for a modded weapon, not here yet).
pub fn row_text(item: &ItemLine) -> String {
    if item.count > 1 {
        format!("{} ({})", item.name, item.count)
    } else {
        item.name.clone()
    }
}

/// The card mask for an item (`00707e30`): weapons 0xc3e (DPS, weight,
/// value, condition, ammunition ("--" without), strength, damage; the
/// damage card stays at the file's alpha 0, so it doesn't show), apparel
/// 0x3c (weight, value, condition, and the ammunition card holding the
/// weight class) with DR or DT (0x1 / 0x1000; "--" in DR with neither),
/// everything else weight and value (0xc), with the effects (0x40) when
/// it has any (aid, ammunition).
pub fn card_mask(item: &ItemLine) -> u32 {
    let mut m = match item.tab {
        ItemTab::Weapons if item.damage.is_some() => 0xc3e,
        ItemTab::Apparel => {
            let mut m = 0x3c;
            let dr = item.damage_resistance.unwrap_or(0.0);
            let dt = item.damage_threshold.unwrap_or(0.0);
            if dr == 0.0 && dt == 0.0 {
                m |= 0x1;
            }
            if dr != 0.0 {
                m |= 0x1;
            }
            if dt != 0.0 {
                m |= 0x1000;
            }
            m
        }
        _ => 0xc,
    };
    if item.effects.as_deref().is_some_and(|e| !e.is_empty()) && item.tab != ItemTab::Weapons {
        m |= 0x40;
    }
    m
}

/// The damage card's text (`00707e30`, the value of `006450f0`): "%d" of
/// the damage rounded to a whole number (`004bd510`: halves up), or with
/// more than one projectile a shot "%.1fx%d" (each one's share × the
/// count).
pub fn damage_text(damage: f32, projectiles: u32) -> String {
    if projectiles < 2 {
        format!("{}", round_half_up(damage))
    } else {
        format!("{:.1}x{}", damage / projectiles as f32, projectiles)
    }
}

/// `004bd510` with a step of 1: the whole part, plus one when what's left
/// is at least a half.
fn round_half_up(v: f32) -> i32 {
    let whole = v.trunc();
    whole as i32 + i32::from(v - whole >= 0.5)
}

/// A count × amount as the card writes it (`00707e30`): "--" for nothing,
/// `%.1f` below 1, else `%.0f` (the value); the weight `%.2f`.
fn value_text(v: f32) -> String {
    if v <= 0.0 {
        "--".into()
    } else if v < 1.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.0}")
    }
}

impl ItemsMenu {
    /// Reads the menu and builds its tab line (`0077fc10`: ids from 24,
    /// `sInventoryWeapons` .. `sInventoryAmmoTab`) and the cards' titles.
    pub fn load(
        ui: &mut Ui,
        read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<ItemsMenu, String> {
        let menu = super::load_menu(ui, super::ITEMS_FILE, read)?;
        let list_tile = by_id(ui, menu, 4).unwrap_or(menu);
        let list = ListBox::new(menu, list_tile, "IM_InventoryListTemplate");
        let tabline_tile = by_id(ui, menu, 13);
        let labels: Vec<String> = [
            "sInventoryWeapons",
            "sInventoryApparel",
            "sInventoryAid",
            "sInventoryMisc",
            "sInventoryAmmoTab",
        ]
        .iter()
        .map(|s| text(ui, s))
        .collect();
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let tabs = match tabline_tile {
            Some(tl) => tabline::build(ui, menu, tl, 0x18, &refs),
            None => Vec::new(),
        };
        let title = trait_id(ui, "_Title");
        for (id, setting) in [
            (14, "sInventoryDamage"),
            (15, "sInventoryDamagePerSecond"),
            (16, "sInventoryStrReq"),
            (17, "sInventoryDamageResistance"),
            (18, "sInventoryDamageThreshold"),
        ] {
            if let Some(tile) = by_id(ui, menu, id) {
                let v = text(ui, setting);
                ui.set_string(tile, title, &v);
            }
        }
        if let Some(cancel) = by_id(ui, menu, 10) {
            let v = text(ui, "sCancel");
            ui.set_string(cancel, t::STRING, &v);
        }
        // The headline's DR card sits where the DT card does: New Vegas
        // shows damage threshold (a guess at which the code hides).
        if let Some(dr) = by_id(ui, menu, 2) {
            ui.set_number(dr, t::VISIBLE, 0.0);
        }
        let mut m = ItemsMenu {
            menu,
            tab: 0,
            list,
            tabline: tabline_tile,
            tabs,
            card: by_id(ui, menu, 12),
            icon: by_id(ui, menu, 11),
            shown: Vec::new(),
            filled: None,
            hovered: None,
            keyring: false,
        };
        if let Some(tl) = m.tabline {
            tabline::set_current(ui, tl, 0);
        }
        m.show_card(ui, None);
        Ok(m)
    }

    /// The headline (`00782a90`): caps, hit points "%d/%d", damage
    /// threshold "%1.1f", weight "%d/%d"; then the tab's rows.
    pub fn fill(&mut self, ui: &mut Ui, input: &PipboyInput) {
        let value = trait_id(ui, "_Value");
        let set = |ui: &mut Ui, id: i32, v: String| {
            if let Some(tile) = by_id(ui, self.menu, id) {
                ui.set_string(tile, value, &v);
            }
        };
        set(ui, 0, format!("{}", input.caps));
        set(
            ui,
            1,
            format!("{}/{}", input.health.0 as i32, input.health.1 as i32),
        );
        set(ui, 2, format!("{}", input.damage_resistance as i32));
        set(ui, 21, format!("{:.1}", input.damage_threshold));
        set(
            ui,
            3,
            format!("{}/{}", input.weight.0 as i32, input.weight.1 as i32),
        );
        let items: Vec<ItemLine> = self.tab_items(input);
        let keyring_row = self.keyring_row(input);
        let now = (self.tab, self.keyring, items.clone(), keyring_row);
        if self.filled.as_ref() != Some(&now) {
            self.fill_rows(ui, &items, keyring_row);
            self.filled = Some(now);
        }
    }

    /// The items in the list (`007824e0`, the list's order: names
    /// compared; the same name by condition, not kept here): the tab's,
    /// or with the keyring open the keys (`00782810`).
    fn tab_items(&self, input: &PipboyInput) -> Vec<ItemLine> {
        let mut items: Vec<ItemLine> = if self.keyring {
            input.keys.clone()
        } else {
            input
                .items
                .iter()
                .filter(|i| i.tab as usize == self.tab)
                .cloned()
                .collect()
        };
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    /// Whether the keyring's row shows: on the Misc tab (`00782620` lets
    /// the row without an item through there only) with keys carried
    /// (`004c6ba0(0x2e)`), the keyring closed; sorted last (`007824e0`).
    fn keyring_row(&self, input: &PipboyInput) -> bool {
        !self.keyring && self.tab == ItemTab::Misc as usize && !input.keys.is_empty()
    }

    fn fill_rows(&mut self, ui: &mut Ui, items: &[ItemLine], keyring_row: bool) {
        let keep = self.list.selected;
        self.list.clear(ui);
        self.shown.clear();
        for item in items {
            let Some(row) = self.list.add(ui, Some(&row_text(item))) else {
                continue;
            };
            // Rows' `id` 0x1d (29), what the click handler looks for.
            ui.set_number(row, t::ID, ROW_ID as f32);
            if let Some(marker) = ui.find_below(row, "IM_Template_ItemMarker") {
                ui.set_number(marker, t::VISIBLE, if item.equipped { 1.0 } else { 0.0 });
            }
            self.shown.push(item.form);
        }
        if keyring_row {
            let name = text(ui, "sKeyring");
            if let Some(row) = self.list.add(ui, Some(&name)) {
                ui.set_number(row, t::ID, KEYRING_ID as f32);
                if let Some(marker) = ui.find_below(row, "IM_Template_ItemMarker") {
                    ui.set_number(marker, t::VISIBLE, 0.0);
                }
                self.shown.push(0);
            }
        }
        let rows = self.shown.len();
        let chosen = (rows > 0).then(|| keep.unwrap_or(0).min(rows - 1));
        self.list.select(ui, chosen);
        self.show_row(ui, chosen, items);
    }

    /// The card for a row: an item's, or the keyring's picture alone.
    fn show_row(&mut self, ui: &mut Ui, row: Option<usize>, items: &[ItemLine]) {
        match row.and_then(|i| self.shown.get(i).copied()) {
            Some(0) => self.show_keyring(ui),
            _ => self.show_card(ui, row.and_then(|i| items.get(i))),
        }
    }

    /// The keyring's row chosen (`00780ff0` case 0x1e): no item chosen
    /// (`00781b10`), its picture shown.
    fn show_keyring(&mut self, ui: &mut Ui) {
        self.show_card(ui, None);
        if let Some(icon) = self.icon {
            ui.set_string(icon, t::FILENAME, KEYRING_PICTURE);
            ui.set_number(icon, t::VISIBLE, 1.0);
        }
    }

    /// Opens or closes the keyring (`00780140` cases 0x1e and 10): the
    /// menu's `_KeyringOpen`, the list's filter, the list made again.
    pub fn set_keyring(&mut self, ui: &mut Ui, open: bool, input: &PipboyInput) {
        self.keyring = open;
        let id = trait_id(ui, "_KeyringOpen");
        ui.set_number(self.menu, id, if open { 1.0 } else { 0.0 });
        self.list.selected = None;
        self.filled = None;
        self.fill(ui, input);
    }

    /// The chosen item's card and picture (`00780ff0` on a row,
    /// `00707e30` the card): `_EquippableItem`, the picture
    /// (`Interface\Icons\` + the item's own), the cards its mask shows.
    fn show_card(&mut self, ui: &mut Ui, item: Option<&ItemLine>) {
        self.update_buttons(ui, item);
        let equippable = trait_id(ui, "_EquippableItem");
        ui.set_number(
            self.menu,
            equippable,
            if item.is_some_and(|i| i.usable) {
                1.0
            } else {
                0.0
            },
        );
        if let Some(icon) = self.icon {
            match item.and_then(|i| i.icon.clone()) {
                Some(path) => {
                    ui.set_string(icon, t::FILENAME, &path);
                    ui.set_number(icon, t::VISIBLE, 1.0);
                }
                None => ui.set_number(icon, t::VISIBLE, 0.0),
            }
        }
        let Some(card) = self.card else {
            return;
        };
        let mask = item.map_or(0, card_mask);
        let title = trait_id(ui, "_Title");
        let value = trait_id(ui, "_Value");
        for (name, bit) in CARDS {
            let Some(tile) = ui.find_below(card, name) else {
                continue;
            };
            ui.set_number(tile, t::VISIBLE, if mask & bit != 0 { 1.0 } else { 0.0 });
            let Some(item) = item else {
                continue;
            };
            let count = item.count.max(1) as f32;
            let v = match name {
                "DamageResistInfo" => item
                    .damage_resistance
                    .filter(|&v| v != 0.0)
                    .map_or("--".into(), |v| format!("{}", v as i32)),
                "DamageThresholdInfo" => item
                    .damage_threshold
                    .map_or("--".into(), |v| format!("{}", v as i32)),
                "DAMInfo" => item
                    .damage
                    .map_or("--".into(), |v| damage_text(v, item.projectiles)),
                // `00645380`'s value, "%d" rounded; none worked out yet:
                // left empty.
                "DPSInfo" => item
                    .dps
                    .map_or(String::new(), |v| format!("{}", round_half_up(v))),
                "StrengthReqInfo" => {
                    // Shown for weapons, made opaque by the code (the
                    // file's alpha is 0): "%d".
                    if mask & bit != 0 {
                        ui.set_number(tile, t::ALPHA, 255.0);
                    }
                    format!("{}", item.strength.unwrap_or(0).max(0))
                }
                "ValueInfo" => value_text(item.value as f32 * count),
                "WeightInfo" => {
                    let w = item.weight * count;
                    if w <= 0.0 {
                        "--".into()
                    } else {
                        format!("{w:.2}")
                    }
                }
                "EffectsInfo" => item.effects.clone().unwrap_or_default(),
                "AmmoInfo" => {
                    // The card's title: a weapon's ammunition ("--"
                    // without), apparel's weight class
                    // (`sArmorWeightLight` / `Medium` / `Heavy`).
                    let text = match (item.tab, item.weight_class) {
                        (ItemTab::Apparel, Some(c)) => super::text(
                            ui,
                            [
                                "sArmorWeightLight",
                                "sArmorWeightMedium",
                                "sArmorWeightHeavy",
                            ][usize::from(c.min(2))],
                        ),
                        _ => item.ammo.clone().unwrap_or_else(|| "--".into()),
                    };
                    ui.set_string(tile, title, &text);
                    continue;
                }
                "ConditionInfo" => {
                    // The condition meter reads the card's `user5`
                    // (at most 1); `user6` 1 for weapons moves the repair
                    // arrows, 0 for apparel.
                    ui.set_number(tile, t::USER0 + 5, item.condition.unwrap_or(1.0).min(1.0));
                    let weapon = item.tab == ItemTab::Weapons;
                    ui.set_number(tile, t::USER0 + 6, if weapon { 1.0 } else { 0.0 });
                    continue;
                }
                _ => continue,
            };
            ui.set_string(tile, value, &v);
        }
        // The effects card's height (`00707e30`): half the card's height
        // less 20, twice that when the condition or ammunition card shows
        // (a second row).
        if let Some(effects) = ui.find_below(card, "EffectsInfo") {
            let mut y = ui.number(card, t::HEIGHT) / 2.0 - 20.0;
            if mask & 0x30 != 0 {
                y += y;
            }
            ui.set_number(effects, t::Y, y);
        }
    }

    /// The buttons' `target`s (their lines brighten with it). Translated
    /// from 00781680 (decompiled, FalloutNV.exe 1.4.0.525),
    /// `InventoryMenu::UpdateButtons` (Xbox PDB): with no item chosen,
    /// Equip (6), Drop (7), Repair (8), Hot key (9) and Mod (19) can't be
    /// pressed; with one, Equip as the item can be equipped or used, Drop
    /// and Mod always, Hot key unless it's ammunition (form type 0x29),
    /// Repair when it can be repaired (`00781860`). (The Equip button's
    /// text, set there too, shows with a pad only: not here.)
    fn update_buttons(&mut self, ui: &mut Ui, item: Option<&ItemLine>) {
        let targets: [(i32, bool); 5] = match item {
            None => [(6, false), (7, false), (8, false), (9, false), (19, false)],
            Some(i) => [
                (6, i.usable),
                (7, true),
                (9, i.tab != ItemTab::Ammo),
                (19, true),
                (8, i.repairable),
            ],
        };
        for (id, on) in targets {
            if let Some(tile) = by_id(ui, self.menu, id) {
                ui.set_number(tile, t::TARGET, if on { 1.0 } else { 0.0 });
            }
        }
    }

    /// Shows a tab (0 Weapons .. 4 Ammo).
    pub fn show_tab(&mut self, ui: &mut Ui, tab: usize, input: &PipboyInput) {
        // Another tab closes the keyring (`00780140` cases 0x18 .. 0x1c).
        if self.keyring {
            self.keyring = false;
            let id = trait_id(ui, "_KeyringOpen");
            ui.set_number(self.menu, id, 0.0);
        }
        self.tab = tab.min(4);
        if let Some(tl) = self.tabline {
            tabline::set_current(ui, tl, self.tab);
        }
        self.list.selected = None;
        self.filled = None;
        self.fill(ui, input);
    }

    /// A key (`00782190`): left and right change tab, wrapping round the
    /// five (the knob turning, `UIPipBoyTab`), up and down choose a row
    /// (`UIPipBoyScroll`), the A button equips, takes off or uses the
    /// chosen item. (Page Up presses Mod and Page Down the hot keys: not
    /// here yet.)
    pub fn key(&mut self, ui: &mut Ui, key: Key, input: &PipboyInput) -> Vec<Action> {
        let mut out = Vec::new();
        match key {
            Key::Left | Key::Right => {
                let tab = if key == Key::Right {
                    (self.tab + 1) % 5
                } else {
                    (self.tab + 4) % 5
                };
                self.show_tab(ui, tab, input);
                out.push(Action::Sound("UIPipBoyTab".into()));
            }
            Key::Up | Key::Down => {
                let before = self.list.selected;
                self.list.step(ui, if key == Key::Down { 1 } else { -1 });
                if self.list.selected != before {
                    let items = self.tab_items(input);
                    self.show_row(ui, self.list.selected, &items);
                    out.push(Action::Sound("UIPipBoyScroll".into()));
                }
            }
            // The A button presses the chosen row: the keyring's opens it.
            Key::Activate
                if self
                    .list
                    .selected
                    .is_some_and(|i| self.shown.get(i) == Some(&0)) =>
            {
                self.set_keyring(ui, true, input);
            }
            Key::Activate => {
                let items = self.tab_items(input);
                if let Some(action) = self
                    .list
                    .selected
                    .and_then(|i| items.get(i))
                    .and_then(|i| self.activate(i))
                {
                    out.push(action);
                }
            }
            _ => {}
        }
        out
    }

    /// What equipping or using a row's item asks of the game (`00780140`
    /// case 0x1d: weapons and apparel are equipped or taken off, the rest
    /// used), when it can be.
    fn activate(&self, item: &ItemLine) -> Option<Action> {
        item.usable.then_some(match item.tab {
            ItemTab::Weapons | ItemTab::Apparel => Action::Equip(item.form),
            _ => Action::Use(item.form),
        })
    }

    /// A tile clicked (`00780140`, slot 0x0c): a tab button (0x18 .. 0x1c)
    /// turns to its tab when it isn't the one shown; a row (0x1d) equips,
    /// takes off or uses its item; Drop (7, the right button on PC) drops
    /// the chosen item (the game's checks and "how many?" are the
    /// caller's). (Repair 8, Cancel 10, Mod 0x13 and the keyring 0x1e are
    /// not here yet.)
    pub fn click(
        &mut self,
        ui: &mut Ui,
        id: i32,
        tile: Option<TileId>,
        input: &PipboyInput,
    ) -> Vec<Action> {
        let mut out = Vec::new();
        match id {
            FIRST_TAB_ID..=0x1c => {
                let tab = (id - FIRST_TAB_ID) as usize;
                if tab != self.tab {
                    self.show_tab(ui, tab, input);
                    out.push(Action::Sound("UIPipBoyTab".into()));
                }
            }
            // Not from the keyring: the refusal sound.
            DROP_ID => {
                let items = self.tab_items(input);
                if let Some(item) = self.list.selected.and_then(|i| items.get(i)) {
                    out.push(if self.keyring {
                        Action::Sound("UIVATSInsufficientAP".into())
                    } else {
                        Action::Drop(item.form)
                    });
                }
            }
            KEYRING_ID if !self.keyring => self.set_keyring(ui, true, input),
            CANCEL_ID if self.keyring => self.set_keyring(ui, false, input),
            ROW_ID => {
                let items = self.tab_items(input);
                let index = tile.and_then(|t| self.list.index_of(t));
                if let Some(action) = index
                    .and_then(|i| items.get(i))
                    .and_then(|i| self.activate(i))
                {
                    out.push(action);
                }
            }
            _ => {}
        }
        out
    }

    /// The pointer onto a tile (`00780ff0`, slot 0x10): a row becomes the
    /// chosen item, its card and picture shown; the knob clicks
    /// (`007f8610`, `UIPipBoyScroll`) when the row's `listindex` differs from
    /// the last one the pointer was over.
    pub fn mouseover(
        &mut self,
        ui: &mut Ui,
        id: i32,
        tile: TileId,
        input: &PipboyInput,
    ) -> Vec<Action> {
        let mut out = Vec::new();
        if id == KEYRING_ID {
            // The keyring's row (`00780ff0` case 0x1e): chosen, its
            // picture; no knob.
            if let Some(index) = self.list.index_of(tile) {
                self.list.choose(ui, Some(index));
                self.show_keyring(ui);
            }
            return out;
        }
        if id != ROW_ID {
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
        let items = self.tab_items(input);
        self.show_card(ui, items.get(index));
        out
    }

    /// The pointer off a tile (`00781620`, slot 0x14; the interface first
    /// lets the list drop its choice, `00717ef0`): leaving a row clears the
    /// card and `_EquippableItem`.
    pub fn unmouseover(&mut self, ui: &mut Ui, id: i32, tile: TileId) {
        if id != ROW_ID && id != KEYRING_ID {
            return;
        }
        if let Some(index) = self.list.index_of(tile) {
            if self.list.selected == Some(index) {
                self.list.choose(ui, None);
            }
        }
        self.show_card(ui, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn item(name: &str, tab: ItemTab) -> ItemLine {
        ItemLine {
            form: 0x100,
            name: name.into(),
            count: 1,
            tab,
            equipped: false,
            usable: true,
            value: 100,
            weight: 1.5,
            icon: None,
            damage: None,
            dps: None,
            projectiles: 1,
            damage_resistance: None,
            damage_threshold: None,
            condition: None,
            strength: None,
            ammo: None,
            weight_class: None,
            effects: None,
            repairable: false,
        }
    }

    #[test]
    fn the_cards_shown_follow_the_item_card_code() {
        let mut gun = item("9mm Pistol", ItemTab::Weapons);
        gun.damage = Some(10.0);
        // Ammunition or not, a weapon's cards are 0xc3e.
        assert_eq!(card_mask(&gun), 0xc3e);
        gun.ammo = Some("9mm (13/11)".into());
        assert_eq!(card_mask(&gun), 0xc3e);
        // Apparel: weight, value, condition, weight class; DR shows "--"
        // with neither DR nor DT, DT only when it has one.
        let mut armour = item("Leather Armor", ItemTab::Apparel);
        armour.damage_resistance = Some(0.0);
        armour.damage_threshold = Some(0.0);
        assert_eq!(card_mask(&armour), 0x3d);
        armour.damage_threshold = Some(6.0);
        assert_eq!(card_mask(&armour), 0x103c);
        // Aid and ammunition with effects add the effects card.
        let mut stimpak = item("Stimpak", ItemTab::Aid);
        assert_eq!(card_mask(&stimpak), 0xc);
        stimpak.effects = Some("Restore Health".into());
        assert_eq!(card_mask(&stimpak), 0x4c);
        assert_eq!(card_mask(&item("Tin Can", ItemTab::Misc)), 0xc);
    }

    #[test]
    fn numbers_are_written_as_the_card_writes_them() {
        // `004bd510`: halves up.
        assert_eq!(damage_text(28.5, 1), "29");
        assert_eq!(damage_text(28.4, 1), "28");
        // "%.1fx%d" with more than one projectile a shot.
        assert_eq!(damage_text(45.0, 9), "5.0x9");
        assert_eq!(value_text(0.0), "--");
        assert_eq!(value_text(0.5), "0.5");
        assert_eq!(value_text(12.4), "12");
        let mut stimpaks = item("Stimpak", ItemTab::Aid);
        assert_eq!(row_text(&stimpaks), "Stimpak");
        stimpaks.count = 3;
        assert_eq!(row_text(&stimpaks), "Stimpak (3)");
    }

    /// `inventory_menu.xml` and `item_stats_display.xml` cut down to the
    /// tiles the code finds by `id` and name.
    const MENU: &str = r#"<menu name="InventoryMenu"><locus>&true;</locus>
      <rect name="caps"><id>0</id></rect><rect name="hp"><id>1</id></rect>
      <rect name="dr"><id>2</id><visible>&true;</visible></rect><rect name="wg"><id>3</id></rect>
      <rect name="dt"><id>21</id></rect>
      <hotrect name="list"><id>4</id><x>0</x><y>100</y><width>300</width><height>400</height>
        <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
        <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
      </hotrect>
      <template name="IM_InventoryListTemplate"><hotrect name="row"><height>30</height>
        <image name="IM_Template_ItemMarker"><visible>&false;</visible></image></hotrect></template>
      <image name="IM_ItemIcon"><id>11</id><visible>&false;</visible></image>
      <rect name="ItemData"><id>12</id><height>100</height>
        <rect name="DamageResistInfo"><visible>&false;</visible></rect>
        <rect name="DPSInfo"><visible>&false;</visible></rect>
        <rect name="WeightInfo"><visible>&false;</visible></rect>
        <rect name="ValueInfo"><visible>&false;</visible></rect>
        <rect name="ConditionInfo"><visible>&false;</visible></rect>
        <rect name="AmmoInfo"><visible>&false;</visible></rect>
        <rect name="EffectsInfo"><visible>&false;</visible></rect>
        <rect name="StrengthReqInfo"><alpha>0</alpha><visible>&true;</visible></rect>
        <rect name="DAMInfo"><alpha>0</alpha><visible>&false;</visible></rect>
        <rect name="DamageThresholdInfo"><visible>&false;</visible></rect>
      </rect>
    </menu>"#;

    fn input() -> PipboyInput {
        let mut gun = item("9mm Pistol", ItemTab::Weapons);
        gun.form = 0x200;
        gun.equipped = true;
        gun.damage = Some(10.0);
        gun.dps = Some(28.5);
        gun.strength = Some(2);
        gun.condition = Some(1.0);
        gun.ammo = Some("9mm (13/11)".into());
        let mut armour = item("Combat Armor", ItemTab::Apparel);
        armour.form = 0x300;
        armour.damage_resistance = Some(0.0);
        armour.damage_threshold = Some(0.0);
        armour.weight_class = Some(1);
        armour.condition = Some(1.0);
        PipboyInput {
            items: vec![armour, gun],
            caps: 250,
            health: (150.0, 200.0),
            damage_threshold: 6.5,
            weight: (17.0, 200.0),
            ..PipboyInput::default()
        }
    }

    #[test]
    fn the_menu_shows_the_headline_the_tab_and_the_chosen_items_card() {
        let mut ui = crate::pipboy::tests::ui();
        let mut read = |p: &str| (p == crate::pipboy::ITEMS_FILE).then(|| MENU.as_bytes().to_vec());
        let mut m = ItemsMenu::load(&mut ui, &mut read).unwrap();
        let input = input();
        m.fill(&mut ui, &input);
        ui.refresh();
        let value = ui.names.lookup("_Value").unwrap();
        let shown = |ui: &mut Ui, id: i32| {
            let tile = by_id(ui, m.menu, id).unwrap();
            ui.string(tile, value).unwrap_or_default()
        };
        assert_eq!(shown(&mut ui, 0), "250");
        assert_eq!(shown(&mut ui, 1), "150/200");
        assert_eq!(shown(&mut ui, 21), "6.5");
        assert_eq!(shown(&mut ui, 3), "17/200");
        // The DR card gives way to DT.
        let dr = by_id(&ui, m.menu, 2).unwrap();
        assert_eq!(ui.number(dr, t::VISIBLE), 0.0);
        // The weapons tab: one row, its marker on (it's equipped).
        assert_eq!(m.shown, [0x200]);
        let marker = ui
            .find_below(m.list.rows[0], "IM_Template_ItemMarker")
            .unwrap();
        assert_eq!(ui.number(marker, t::VISIBLE), 1.0);
        let card = by_id(&ui, m.menu, 12).unwrap();
        let tile = |ui: &Ui, name: &str| ui.find_below(card, name).unwrap();
        let title = ui.names.lookup("_Title").unwrap();
        let dps = tile(&ui, "DPSInfo");
        assert_eq!(ui.number(dps, t::VISIBLE), 1.0);
        assert_eq!(ui.string(dps, value).unwrap(), "29");
        // Strength made opaque, the damage card left at the file's alpha 0.
        let strength = tile(&ui, "StrengthReqInfo");
        assert_eq!(ui.number(strength, t::ALPHA), 255.0);
        assert_eq!(ui.string(strength, value).unwrap(), "2");
        let dam = tile(&ui, "DAMInfo");
        assert_eq!(ui.number(dam, t::ALPHA), 0.0);
        assert_eq!(ui.string(dam, value).unwrap(), "10");
        let ammo = tile(&ui, "AmmoInfo");
        assert_eq!(ui.string(ammo, title).unwrap(), "9mm (13/11)");
        assert_eq!(ui.number(tile(&ui, "ConditionInfo"), t::USER0 + 6), 1.0);
        // The effects card: half the card's height less 20, twice that
        // under a second row.
        assert_eq!(ui.number(tile(&ui, "EffectsInfo"), t::Y), 60.0);
        // Activating equips; right turns to Apparel.
        assert_eq!(
            m.key(&mut ui, Key::Activate, &input),
            [Action::Equip(0x200)]
        );
        assert_eq!(
            m.key(&mut ui, Key::Right, &input),
            [Action::Sound("UIPipBoyTab".into())]
        );
        ui.refresh();
        assert_eq!(m.shown, [0x300]);
        assert_eq!(ui.string(ammo, title).unwrap(), "Medium");
        let dr_card = tile(&ui, "DamageResistInfo");
        assert_eq!(ui.number(dr_card, t::VISIBLE), 1.0);
        assert_eq!(ui.string(dr_card, value).unwrap(), "--");
        assert_eq!(ui.number(tile(&ui, "ConditionInfo"), t::USER0 + 6), 0.0);
        assert_eq!(ui.number(tile(&ui, "DPSInfo"), t::VISIBLE), 0.0);
    }
}
