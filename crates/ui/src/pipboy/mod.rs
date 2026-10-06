//! The Pip-Boy 3000: the game's three Pip-Boy menus (`menus\main\
//! stats_menu.xml` STATS, `inventory_menu.xml` ITEMS, `map_menu.xml` DATA)
//! read and worked out by the menu system, and filled the way
//! FalloutNV.exe's menu classes fill them (`StatsMenu` vtable `0106ffd4`,
//! made by `007da2c0`; `InventoryMenu` `010739b4`, `0077fc10`; `MapMenu`
//! `01074d44`, `00796b90`): the code finds its tiles by their `id`, writes
//! the values the files read through `io()` (`user5` the health text, ...),
//! makes list rows and tab buttons from the files' templates, and moves
//! between pages as keys come in.
//!
//! With `[Pipboy] bUsePipboyMode` 1 (this install) the menus aren't drawn
//! on the screen: they're drawn into a picture (1280 × 960 menu units,
//! `007fba00`) shown on the Pip-Boy model's `pipboyscreen` (see
//! [`screen`] for how that picture is made to look like a screen).
//!
//! The game state comes in as plain values ([`PipboyInput`]; `gather`
//! reads them from the engine's state).

pub mod data;
pub mod gather;
pub mod items;
pub mod screen;
pub mod stats;

use crate::menu::{Interface, MenuCode};
use crate::names::t;
use crate::tile::{TileId, Ui};

pub use data::DataMenu;
pub use items::ItemsMenu;
pub use stats::StatsMenu;

/// The three menus' files.
pub const STATS_FILE: &str = "menus\\main\\stats_menu.xml";
pub const ITEMS_FILE: &str = "menus\\main\\inventory_menu.xml";
pub const DATA_FILE: &str = "menus\\main\\map_menu.xml";

/// The menus' picture: the orthographic camera the rendered-menu object
/// draws them with (`007fba00`: frustum 0 .. 1280 across, 0 .. 960 down,
/// near 0, far 10000), in menu units. The Pip-Boy's screen mesh shows the
/// top left of it (its texture coordinates run 0 .. 0.75 across and 0 ..
/// 0.76 down), and the mouse's place on the screen maps back through it
/// (`007f8720`: u × 960 × 4/3, v × 960).
pub const PICTURE_SIZE: [f32; 2] = [1280.0, 960.0];

/// A stat line: a SPECIAL, a skill, a perk, a misc statistic.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StatLine {
    pub name: String,
    /// Shown right of the name (`user1`; none: hidden, -1 in the file).
    pub value: Option<i32>,
    pub description: String,
    /// Its picture (`ICON`), as written.
    pub icon: Option<String>,
}

/// A reputation (`REPU`): its name, title now, and picture.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReputationLine {
    pub name: String,
    pub title: String,
    pub icon: Option<String>,
}

/// Which tab of the ITEMS menu an item is under (the tab line's order:
/// Weapons, Apparel, Aid, Misc, Ammo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemTab {
    Weapons = 0,
    Apparel = 1,
    Aid = 2,
    Misc = 3,
    Ammo = 4,
}

/// An item carried.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemLine {
    /// Its form ID (for the caller to act on).
    pub form: u32,
    pub name: String,
    pub count: i32,
    pub tab: ItemTab,
    pub equipped: bool,
    /// Can be equipped or used (`_EquippableItem`).
    pub usable: bool,
    pub value: i32,
    pub weight: f32,
    /// `ICON`, as written.
    pub icon: Option<String>,
    /// The item card's numbers, those it has. (`dps`: `00645380`'s value,
    /// not worked out yet; `gather` leaves it out.)
    pub damage: Option<f32>,
    pub dps: Option<f32>,
    /// A weapon's projectiles a shot (the DPS card writes "%.1fx%d" when
    /// more than one).
    pub projectiles: u32,
    pub damage_resistance: Option<f32>,
    pub damage_threshold: Option<f32>,
    /// 0 to 1.
    pub condition: Option<f32>,
    pub strength: Option<i32>,
    /// "Ammo name (in clip/rest)".
    pub ammo: Option<String>,
    /// Apparel's weight class: 0 light, 1 medium (`BMDT` general flag
    /// 0x08), 2 heavy (0x80).
    pub weight_class: Option<u8>,
    pub effects: Option<String>,
}

/// A quest in the DATA menu.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestLine {
    pub form: u32,
    pub name: String,
    pub completed: bool,
    /// The active quest (`ForceActiveQuest`, or chosen here).
    pub active: bool,
    /// Its objectives shown, and whether each is done.
    pub objectives: Vec<(String, bool)>,
}

/// A note (`NOTE` added to the Pip-Boy).
#[derive(Debug, Clone, PartialEq)]
pub struct NoteLine {
    pub name: String,
    pub text: String,
}

/// A map marker on the world map.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkerLine {
    pub form: u32,
    pub name: String,
    /// Where on the map picture, 0 to 1 across and down (`0079c380`).
    pub at: [f32; 2],
    /// `TNAM` (1 city .. 14 vault).
    pub kind: u8,
    /// Can be travelled to (found).
    pub travel: bool,
}

/// The world map: the worldspace's picture (`ICON`), its usable size
/// (`MNAM`), markers, and the player's place and heading.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldMapLine {
    pub picture: String,
    pub size: [f32; 2],
    pub markers: Vec<MarkerLine>,
    pub player: Option<([f32; 2], f32)>,
}

/// What the Pip-Boy shows, from the game's state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PipboyInput {
    pub name: String,
    pub level: i32,
    /// The XP text: "XP/next level" (`%d/%d`), or none at the top level
    /// (the code writes `sStatsXPMax`).
    pub xp: Option<(i32, i32)>,
    pub health: (f32, f32),
    pub action_points: (f32, f32),
    /// Head, torso, left arm, right arm, left leg, right leg (actor values
    /// 25 to 30), 0 to 100.
    pub limbs: [f32; 6],
    pub rads: f32,
    pub rad_resistance: f32,
    pub hardcore: bool,
    pub effects: Vec<(String, String)>,
    pub stimpaks: i32,
    pub doctors_bags: i32,
    pub radaway: i32,
    pub radx: i32,
    /// The aid items' forms (Stimpak, Doctor's Bag, RadAway, Rad-X: default
    /// objects 0, 21, 3, 2), for the Status page's buttons, and their names
    /// (the buttons' `user0`, `007da2c0`).
    pub aid: [Option<u32>; 4],
    pub aid_names: [String; 4],
    pub special: Vec<StatLine>,
    pub skills: Vec<StatLine>,
    pub perks: Vec<StatLine>,
    pub general: Vec<StatLine>,
    /// Karma: 0 good, 1 neutral, 2 bad, 3 very good, 4 very evil (the
    /// order of `007dd090`'s pictures), the alignment's name and the
    /// karmic title.
    pub karma_band: u8,
    pub alignment: String,
    pub karma_title: String,
    pub reputations: Vec<ReputationLine>,
    pub items: Vec<ItemLine>,
    pub caps: i32,
    pub weight: (f32, f32),
    pub damage_resistance: f32,
    pub damage_threshold: f32,
    pub location: String,
    pub date_time: String,
    pub quests: Vec<QuestLine>,
    pub notes: Vec<NoteLine>,
    pub world_map: Option<WorldMapLine>,
    pub stations: Vec<String>,
}

/// The three Pip-Boy menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Stats,
    Items,
    Data,
}

/// A key as the menus see it: the codes the interface manager hands the
/// menus' key handlers (`0070f6e0` → `00717f80` → the menu's slot 0x38,
/// `007db680` and kin). On a PC keyboard (`007154b0`, `0070c4a0`): the
/// arrows give 1 up, 2 down, 4 left, 3 right, and with Shift held left and
/// right give 0x0D / 0x0E (the previous / next menu, the pad's triggers);
/// Enter gives the A button (-2: the chosen tile), with Shift the X
/// button (0x0B), with Alt the Y button (0x0C); Page Up / Page Down the
/// bumpers (0x0F / 0x10); other keys their letter, looked up as the menu's
/// `_PCButton_<letter>` trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    /// The next page or tab (code 3).
    Right,
    /// The previous one (code 4).
    Left,
    /// The next or previous menu (STATS, ITEMS, DATA: codes 0x0E / 0x0D).
    NextSection,
    PrevSection,
    /// The A button (Enter): equip, use, travel, make active.
    Activate,
    /// The X and Y buttons (Shift + Enter, Alt + Enter).
    ButtonX,
    ButtonY,
    /// A letter: the button the menu's `_PCButton_<letter>` names
    /// (`stats_stimpak_button` for S), clicked when it shows and can be
    /// clicked (`0070c4a0`).
    Letter(char),
}

/// What the Pip-Boy asks of the game.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Play a sound record (by editor ID).
    Sound(String),
    /// Equip or take off an item, or use it (aid, books).
    Equip(u32),
    Use(u32),
    /// Fast travel to a map marker (its reference).
    Travel(u32),
    /// Make a quest the active one.
    ActiveQuest(u32),
}

/// A tile found by its `id` in a menu (the menu objects keep their tiles
/// by id: `StatsMenu` 60 of them from `+0x90`, checked by `007db340`).
pub fn by_id(ui: &Ui, root: TileId, id: i32) -> Option<TileId> {
    let mut stack = vec![root];
    while let Some(tile) = stack.pop() {
        if ui.tiles[tile]
            .traits
            .get(&t::ID)
            .is_some_and(|tr| tr.value.number == id as f32 && tr.actions.is_empty())
        {
            return Some(tile);
        }
        for &c in ui.tiles[tile].children.iter().rev() {
            stack.push(c);
        }
    }
    None
}

/// A custom trait's number.
pub(crate) fn trait_id(ui: &mut Ui, name: &str) -> i32 {
    ui.names.lookup_or_add(name).unwrap_or(0)
}

/// A setting's text.
pub(crate) fn text(ui: &Ui, name: &str) -> String {
    ui.setting_text(name).unwrap_or_default()
}

/// Reads a menu file (with its prefabs) and puts it on the screen hidden.
pub(crate) fn load_menu(
    ui: &mut Ui,
    file: &str,
    read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
) -> Result<TileId, String> {
    let bytes = read(file).ok_or_else(|| format!("{file} not found"))?;
    ui.load_menu(&bytes, read)
        .map_err(|e| format!("{file}: {e}"))
}

/// The Pip-Boy: its three menus and which shows.
pub struct Pipboy {
    pub stats: StatsMenu,
    pub items: ItemsMenu,
    pub data: DataMenu,
    pub section: Section,
    /// The interface's pointer state over the shown menu (the tile under
    /// the pointer, the one the button went down on, a drag).
    pub interface: Interface,
}

/// The menu classes' numbers (`Menu` vtable slot `+0x34`): StatsMenu
/// 1003 (0x3eb), InventoryMenu 1002 (0x3ea), MapMenu 1023 (0x3ff), as
/// `00717920` and `00704c10` / `007048f0` / `00704170` name them.
pub const STATS_CLASS: i32 = 0x3eb;
pub const ITEMS_CLASS: i32 = 0x3ea;
pub const DATA_CLASS: i32 = 0x3ff;

/// The left mouse button this frame, as `0070c4a0` reads it
/// (`00a23a50(0, 0 / 1 / 2)`: held, gone down, come up).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Button {
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

/// The shown menu's code as the interface sees it (the `Menu` vtable's
/// click, mouse-over and mouse-off slots), with what it asks of the game.
struct Code<'a> {
    section: Section,
    stats: &'a mut StatsMenu,
    items: &'a mut ItemsMenu,
    data: &'a mut DataMenu,
    input: &'a PipboyInput,
    actions: Vec<Action>,
}

impl MenuCode for Code<'_> {
    fn class(&self) -> i32 {
        match self.section {
            Section::Stats => STATS_CLASS,
            Section::Items => ITEMS_CLASS,
            Section::Data => DATA_CLASS,
        }
    }

    fn click(&mut self, ui: &mut Ui, id: i32, tile: Option<TileId>, _now: f64) {
        let input = self.input;
        let out = match self.section {
            Section::Stats => self.stats.click(ui, id, input),
            Section::Items => self.items.click(ui, id, tile, input),
            Section::Data => self.data.click(ui, id, tile, input),
        };
        self.actions.extend(out);
    }

    fn mouseover(&mut self, ui: &mut Ui, id: i32, tile: TileId) {
        let input = self.input;
        let out = match self.section {
            Section::Stats => self.stats.mouseover(ui, tile, input),
            Section::Items => self.items.mouseover(ui, id, tile, input),
            Section::Data => self.data.mouseover(ui, id, tile, input),
        };
        self.actions.extend(out);
    }

    fn unmouseover(&mut self, ui: &mut Ui, id: i32, tile: TileId) {
        if self.section == Section::Items {
            self.items.unmouseover(ui, id, tile);
        }
    }
}

impl Pipboy {
    /// Reads the three menus and sets them up as their makers do.
    pub fn load(
        ui: &mut Ui,
        read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<Pipboy, String> {
        let stats = StatsMenu::load(ui, read)?;
        let items = ItemsMenu::load(ui, read)?;
        let data = DataMenu::load(ui, read)?;
        let mut p = Pipboy {
            stats,
            items,
            data,
            section: Section::Stats,
            interface: Interface::default(),
        };
        p.show(ui, Section::Stats);
        Ok(p)
    }

    /// The shown menu's tile.
    pub fn menu(&self) -> TileId {
        match self.section {
            Section::Stats => self.stats.menu,
            Section::Items => self.items.menu,
            Section::Data => self.data.menu,
        }
    }

    /// Shows one of the three (the others hidden). The pointer's tiles
    /// were the old menu's: the interface lets them go (the menu that
    /// closes takes its tiles with it) and picks afresh.
    pub fn show(&mut self, ui: &mut Ui, section: Section) {
        if section != self.section {
            for tile in [self.interface.over, self.interface.focus]
                .into_iter()
                .flatten()
            {
                ui.set_number(tile, t::MOUSEOVER, 0.0);
            }
            self.interface = Interface::default();
            self.data.cursor_hidden = false;
        }
        self.section = section;
        for (menu, on) in [
            (self.stats.menu, section == Section::Stats),
            (self.items.menu, section == Section::Items),
            (self.data.menu, section == Section::Data),
        ] {
            ui.set_number(menu, t::VISIBLE, if on { 1.0 } else { 0.0 });
        }
    }

    /// Fills all three from the game's state.
    pub fn fill(&mut self, ui: &mut Ui, input: &PipboyInput) {
        self.stats.fill(ui, input);
        self.items.fill(ui, input);
        self.data.fill(ui, input);
        ui.refresh();
    }

    /// A key: moves within the shown menu or to the next one. Returns what
    /// the game should do (sounds, equipping, travel).
    pub fn key(&mut self, ui: &mut Ui, key: Key, input: &PipboyInput) -> Vec<Action> {
        let mut out = Vec::new();
        match key {
            // Round the three (`007db680`, `00782190`, `00799790`: STATS
            // goes back to DATA and on to ITEMS, DATA on to STATS).
            Key::NextSection | Key::PrevSection => {
                let order = [Section::Stats, Section::Items, Section::Data];
                let at = order.iter().position(|&s| s == self.section).unwrap_or(0);
                let next = if key == Key::NextSection {
                    (at + 1) % 3
                } else {
                    (at + 2) % 3
                };
                self.show(ui, order[next]);
                // The tab knob turning (`007fa0f0`).
                out.push(Action::Sound("UIPipBoyTab".into()));
            }
            Key::Letter(c) => {
                if let Some(id) = self.pc_button(ui, c) {
                    out.extend(self.click(ui, id, input));
                }
            }
            _ => match self.section {
                Section::Stats => out.extend(self.stats.key(ui, key, input)),
                Section::Items => out.extend(self.items.key(ui, key, input)),
                Section::Data => out.extend(self.data.key(ui, key, input)),
            },
        }
        ui.refresh();
        out
    }

    /// The `id` of the button a letter presses (`0070c4a0`): the shown
    /// menu's `_PCButton_<letter>` trait names a tile; it's pressed when it
    /// shows (`visible`) and can be (`target`).
    fn pc_button(&self, ui: &mut Ui, c: char) -> Option<i32> {
        let menu = self.menu();
        let name = format!("_PCButton_{}", c.to_ascii_uppercase());
        let trait_id = ui.names.lookup(&name)?;
        let tile_name = ui.string(menu, trait_id)?;
        let tile = ui.find(menu, tile_name.trim())?;
        (ui.number(tile, t::VISIBLE) != 0.0 && ui.number(tile, t::TARGET) != 0.0)
            .then(|| ui.number(tile, t::ID) as i32)
    }

    /// A button of the shown menu pressed (the menus' click handlers, slot
    /// 0x0C: `007db380` STATS, `00780140` ITEMS, `00796fd0` DATA). ITEMS'
    /// lettered buttons (Repair, Mod, the keyring's Cancel) and DATA's (R:
    /// `MM_ButtonY`) do what isn't here yet.
    pub fn click(&mut self, ui: &mut Ui, id: i32, input: &PipboyInput) -> Vec<Action> {
        match self.section {
            Section::Stats => self.stats.click(ui, id, input),
            Section::Items => self.items.click(ui, id, None, input),
            Section::Data => self.data.click(ui, id, None, input),
        }
    }

    /// Runs the interface over the shown menu with its code.
    fn with_code<R>(
        &mut self,
        input: &PipboyInput,
        run: impl FnOnce(&mut Interface, &mut Code, TileId) -> R,
    ) -> (R, Vec<Action>) {
        let menu = self.menu();
        let mut interface = std::mem::take(&mut self.interface);
        let mut code = Code {
            section: self.section,
            stats: &mut self.stats,
            items: &mut self.items,
            data: &mut self.data,
            input,
            actions: Vec::new(),
        };
        let r = run(&mut interface, &mut code, menu);
        let actions = std::mem::take(&mut code.actions);
        self.interface = interface;
        (r, actions)
    }

    /// The mouse over the Pip-Boy (`0070c4a0` with the Pip-Boy up): `at` is
    /// the pointer on the menus' picture in menu units (`007f8720`: the
    /// screen's texture coordinates × 1280 × 960), or none when it's off
    /// the screen, when no tile is under it (`007126c0` gives none with a
    /// Pip-Boy menu on top and the cursor off the screen). Then the
    /// interface's own rules (`ui::menu::Interface::pointer`): the tile
    /// under it moused over, a press and release on the same tile a click,
    /// a `draggable` tile dragged. On DATA's map tabs the highlight box
    /// follows the pointer (`0079a130`). Returns what the game should do
    /// (sounds, the interface's `clicksound` / `mouseoversound` included).
    pub fn pointer(
        &mut self,
        ui: &mut Ui,
        at: Option<[f32; 2]>,
        button: Button,
        now: f64,
        input: &PipboyInput,
    ) -> Vec<Action> {
        if button.pressed && self.section == Section::Data {
            self.data.pressed(ui);
        }
        // Off the screen nothing is picked: a point no tile covers.
        let [x, y] = at.unwrap_or([-1.0e6, -1.0e6]);
        let ((), mut out) = self.with_code(input, |interface, code, menu| {
            interface.pointer(
                ui,
                menu,
                code,
                x,
                y,
                button.down,
                button.pressed,
                button.released,
                now,
            );
        });
        if self.section == Section::Data {
            out.extend(self.data.pointer_moved(ui, at, input));
        }
        out.extend(self.interface_sounds());
        ui.refresh();
        out
    }

    /// The mouse wheel turned `notches` (away from the player positive)
    /// over the Pip-Boy (`0070c4a0`: the nearest `wheelable` tile from the
    /// one under the pointer gets `wheelmoved`, the list boxes' scroll bars
    /// read it).
    pub fn wheel(&mut self, ui: &mut Ui, notches: i32, input: &PipboyInput) -> Vec<Action> {
        let ((), mut out) = self.with_code(input, |interface, code, menu| {
            interface.wheel(ui, menu, code, notches);
        });
        out.extend(self.interface_sounds());
        ui.refresh();
        out
    }

    /// The interface's own sounds (`clicksound`, `mouseoversound`).
    fn interface_sounds(&mut self) -> Vec<Action> {
        self.interface
            .effects
            .drain(..)
            .map(|e| match e {
                crate::menu::Effect::Sound(name) => Action::Sound(name),
            })
            .collect()
    }

    /// One of the Pip-Boy's own buttons pressed with the mouse (`007f8720`
    /// picks `PipBoyButton01` .. `03` on the model: 1 STATS, 2 ITEMS, 3
    /// DATA; `0070c4a0` then plays `UIMenuMode` and opens that menu,
    /// `00704c10` / `007048f0` / `00704170`).
    pub fn press_section(&mut self, ui: &mut Ui, button: usize) -> Vec<Action> {
        let section = match button {
            1 => Section::Stats,
            2 => Section::Items,
            3 => Section::Data,
            _ => return Vec::new(),
        };
        self.show(ui, section);
        ui.refresh();
        vec![Action::Sound("UIMenuMode".into())]
    }

    /// Whether the game's cursor is hidden now (DATA's map under it).
    pub fn cursor_hidden(&self) -> bool {
        self.section == Section::Data && self.data.cursor_hidden
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::tile::{Screen, SystemColors};

    pub fn ui() -> Ui {
        crate::game::new_ui(
            &mut |_| None,
            &|_: &str, _: &str| None,
            std::collections::HashMap::new(),
            1920,
            1080,
        )
    }

    #[test]
    fn tiles_by_id_ignore_ones_worked_out() {
        let mut ui = Ui::new(
            Screen {
                width_px: 1920,
                height_px: 1080,
                safe_x: 15.0,
                safe_y: 15.0,
            },
            SystemColors::new(None, None),
            Box::new(|_| None),
        );
        let m = ui
            .load_menu(
                b"<menu name=\"m\"><rect name=\"a\"><id>5</id></rect><rect name=\"b\"><id><copy>5</copy></id></rect>
                  <rect name=\"c\"><id>0</id></rect></menu>",
                &mut |_| None,
            )
            .unwrap();
        assert_eq!(by_id(&ui, m, 5), ui.find(m, "a"));
        assert_eq!(by_id(&ui, m, 0), ui.find(m, "c"));
        assert_eq!(by_id(&ui, m, 9), None);
    }

    /// `inventory_menu.xml` cut down, laid out for the pointer: rows 30
    /// high from y 100 (their `y` from `_y`, as `list_box_template.xml`
    /// places them), a tab line at y 600 of buttons 100 wide.
    const ITEMS: &str = r#"<menu name="InventoryMenu"><locus>&true;</locus>
      <hotrect name="list"><id>4</id><x>0</x><y>100</y><width>300</width><height>400</height><locus>&true;</locus>
        <target>&false;</target><wheelable>&true;</wheelable>
        <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
        <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
      </hotrect>
      <template name="IM_InventoryListTemplate"><hotrect name="row"><height>30</height><width>300</width>
        <y><copy src="me()" trait="_y"/></y><target>&true;</target></hotrect></template>
      <rect name="tabs"><id>13</id><x>0</x><y>600</y><width>855</width><locus>&true;</locus></rect>
      <template name="TabButtonTemplate"><hotrect name="TabButton"><width>100</width><height>30</height>
        <x><copy src="me()" trait="_x"/></x><mouseoversound>UIMenuFocus</mouseoversound></hotrect></template>
      <image name="IM_ItemIcon"><id>11</id><visible>&false;</visible></image>
    </menu>"#;

    /// `map_menu.xml` cut down: the world map in its clip window, the
    /// highlight box in its own, a marker template placed by `_x` / `_y`.
    const DATA: &str = r#"<menu name="MapMenu"><locus>&true;</locus>
      <hotrect name="MM_WorldMap_ClipWindow"><y>50</y><width>855</width><height>500</height><locus>&true;</locus><target>&false;</target>
        <hotrect name="world"><id>4</id><draggable>&true;</draggable><locus>&true;</locus><width>855</width><height>500</height>
          <_Magnification>1</_Magnification></hotrect>
      </hotrect>
      <hotrect name="MM_Highlight_ClipWindow"><y>50</y><width>855</width><height>500</height><locus>&true;</locus><target>&false;</target>
        <rect name="box"><id>6</id><width>160</width><height>160</height><x>348</x><y>170</y><locus>&true;</locus></rect>
      </hotrect>
      <template name="MapMarkerTemplate"><image name="MapMarker"><user0>1</user0><width>20</width><height>20</height><target>&false;</target>
        <x><copy src="me()" trait="_x"/><mul src="parent()" trait="width"/><sub>10</sub></x>
        <y><copy src="me()" trait="_y"/><mul src="parent()" trait="height"/><sub>10</sub></y></image></template>
    </menu>"#;

    fn load() -> (Ui, Pipboy) {
        let mut ui = ui();
        let mut read = |p: &str| {
            let text = match p {
                STATS_FILE => stats::tests::MENU,
                ITEMS_FILE => ITEMS,
                DATA_FILE => DATA,
                _ => return None,
            };
            Some(text.as_bytes().to_vec())
        };
        let p = Pipboy::load(&mut ui, &mut read).unwrap();
        (ui, p)
    }

    fn input() -> PipboyInput {
        let item = |form: u32, name: &str, tab: ItemTab| ItemLine {
            form,
            name: name.into(),
            count: 1,
            tab,
            equipped: false,
            usable: true,
            value: 10,
            weight: 1.0,
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
        };
        let marker = |form: u32, name: &str, at: [f32; 2]| MarkerLine {
            form,
            name: name.into(),
            at,
            kind: 1,
            travel: true,
        };
        PipboyInput {
            items: vec![
                item(0xA1, "Knife", ItemTab::Weapons),
                item(0xA2, "Pistol", ItemTab::Weapons),
                item(0xB1, "Vault Suit", ItemTab::Apparel),
            ],
            world_map: Some(WorldMapLine {
                picture: "map.dds".into(),
                size: [855.0, 500.0],
                markers: vec![
                    marker(0x901, "Goodsprings", [0.25, 0.75]),
                    marker(0x902, "Primm", [0.75, 0.25]),
                ],
                player: Some(([0.5, 0.5], 0.0)),
            }),
            ..stats::tests::input()
        }
    }

    const UP: Button = Button {
        down: false,
        pressed: false,
        released: false,
    };
    const PRESS: Button = Button {
        down: true,
        pressed: true,
        released: false,
    };
    const RELEASE: Button = Button {
        down: false,
        pressed: false,
        released: true,
    };

    fn sound(name: &str) -> Action {
        Action::Sound(name.into())
    }

    /// `0070c4a0` over ITEMS: the row under the pointer is chosen and its
    /// card shown (`00780ff0`, the knob clicking when the row changes),
    /// a press and release on it equips it (`00780140` case 0x1d), leaving
    /// it clears the choice (`00781620`), a tab button turns the tab
    /// (cases 0x18 .. 0x1c), and the pointer off the screen picks nothing.
    #[test]
    fn the_mouse_chooses_equips_and_turns_tabs_on_items() {
        let (mut ui, mut p) = load();
        let input = input();
        p.fill(&mut ui, &input);
        p.show(&mut ui, Section::Items);
        ui.refresh();
        // The second row (y 130 .. 160 under the list at 100).
        let out = p.pointer(&mut ui, Some([10.0, 145.0]), UP, 0.0, &input);
        assert_eq!(out, [sound("UIPipBoyScroll")]);
        assert_eq!(p.items.list.selected, Some(1));
        // Pressed and let go over it: the pistol is equipped.
        p.pointer(&mut ui, Some([10.0, 145.0]), PRESS, 0.0, &input);
        let out = p.pointer(&mut ui, Some([10.0, 145.0]), RELEASE, 0.0, &input);
        assert_eq!(out, [Action::Equip(0xA2)]);
        // Off the screen: nothing under it, the choice let go.
        p.pointer(&mut ui, None, UP, 0.0, &input);
        assert_eq!(p.interface.over, None);
        assert_eq!(p.items.list.selected, None);
        // The same row again: no knob click (it was the last one).
        let out = p.pointer(&mut ui, Some([10.0, 150.0]), UP, 0.0, &input);
        assert!(out.is_empty());
        // The Apparel tab: gap trunc((855 - 500) / 6) = 59, so its button
        // spans 218 .. 318 along y 600 .. 630.
        // (Its `mouseoversound` from the file as the pointer comes onto it;
        // the knob, `00782470` → `007fa0f0`, as it's clicked.)
        let out = p.pointer(&mut ui, Some([250.0, 610.0]), UP, 0.0, &input);
        assert_eq!(out, [sound("UIMenuFocus")]);
        p.pointer(&mut ui, Some([250.0, 610.0]), PRESS, 0.0, &input);
        let out = p.pointer(&mut ui, Some([250.0, 610.0]), RELEASE, 0.0, &input);
        assert_eq!(out, [sound("UIPipBoyTab")]);
        assert_eq!(p.items.tab, 1);
        assert_eq!(p.items.shown, [0xB1]);
    }

    /// `007f8720` / `0070c4a0`: the model's DATA button shows DATA with
    /// `UIMenuMode`, and the pointer's tiles of the old menu are let go.
    /// Over the world map (`0079a130`, `00799dc0`) the highlight box
    /// follows the pointer, the game's cursor hides, the nearest marker's
    /// name shows and a click on the map travels there.
    #[test]
    fn the_models_buttons_and_the_world_maps_markers() {
        let (mut ui, mut p) = load();
        let input = input();
        p.fill(&mut ui, &input);
        p.pointer(&mut ui, Some([10.0, 10.0]), UP, 0.0, &input);
        assert_eq!(p.press_section(&mut ui, 3), [sound("UIMenuMode")]);
        assert_eq!(p.section, Section::Data);
        assert_eq!(p.interface.over, None);
        ui.refresh();
        // Goodsprings is at (0.25 × 855, 0.75 × 500) on the map, which
        // sits at the clip window's corner (the player centred at 427.5,
        // 250), the window at y 50: (213.75, 425) on the screen.
        let out = p.pointer(&mut ui, Some([220.0, 420.0]), UP, 0.0, &input);
        assert_eq!(out, [sound("UIPipBoyHighlight")]);
        assert_eq!(p.data.marker, Some(0));
        assert!(p.cursor_hidden());
        let bx = by_id(&ui, p.data.menu, 6).unwrap();
        assert_eq!(
            (ui.number(bx, t::X), ui.number(bx, t::Y)),
            (220.0 - 80.0, 370.0 - 80.0)
        );
        let title = ui.names.lookup("_Title").unwrap();
        assert_eq!(ui.string(bx, title).unwrap(), "Goodsprings");
        p.pointer(&mut ui, Some([220.0, 420.0]), PRESS, 0.0, &input);
        let out = p.pointer(&mut ui, Some([220.0, 420.0]), RELEASE, 0.0, &input);
        assert_eq!(out, [Action::Travel(0x901)]);
        // Far from both markers: none; below the map the cursor shows.
        p.pointer(&mut ui, Some([427.0, 300.0]), UP, 0.0, &input);
        assert_eq!(p.data.marker, None);
        p.pointer(&mut ui, Some([427.0, 700.0]), UP, 0.0, &input);
        assert!(!p.cursor_hidden());
    }

    /// `007db380` → `007e0060`: the Status page's RAD button (0x1d) makes
    /// RAD the shown mode; a mode past EFF needs hardcore.
    #[test]
    fn the_status_pages_mode_buttons() {
        let (mut ui, mut p) = load();
        let input = input();
        p.fill(&mut ui, &input);
        p.click(&mut ui, 0x1d, &input);
        assert_eq!(p.stats.status_mode, 1);
        p.click(&mut ui, 0x35, &input);
        assert_eq!(p.stats.status_mode, 1);
    }
}
