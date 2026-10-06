//! DATA (`map_menu.xml`, the `MapMenu` class): five tabs (Local Map,
//! World Map, Quests, Misc, Radio), the place and time along the top.
//! Filled as `00796b90` (setup), `0079cdb0` (the world map and its
//! markers) and the list code fill it.
//!
//! Not here yet: the local map (the game renders the place from above into
//! a picture), quest markers and waypoints on the map, notes' audio,
//! challenges, the radio's stations playing, and the player's arrow turning
//! with the heading (the menus' `rotateangle` isn't drawn).

use super::{by_id, text, trait_id, Action, Key, PipboyInput, QuestLine};
use crate::listbox::ListBox;
use crate::names::t;
use crate::tabline;
use crate::tile::{TileId, Ui};

/// The map markers' pictures by `TNAM` (the table at `011a0404`: 1 city
/// .. 14 vault), and an undiscovered one's.
pub const MARKER_PICTURES: [&str; 15] = [
    "",
    "Interface\\Icons\\World Map\\icon_map_city.dds",
    "Interface\\Icons\\World Map\\icon_map_settlement.dds",
    "Interface\\Icons\\World Map\\icon_map_encampment.dds",
    "Interface\\Icons\\World Map\\icon_map_natural_landmark.dds",
    "Interface\\Icons\\World Map\\icon_map_cave.dds",
    "Interface\\Icons\\World Map\\icon_map_factory.dds",
    "Interface\\Icons\\World Map\\icon_map_monument.dds",
    "Interface\\Icons\\World Map\\icon_map_military.dds",
    "Interface\\Icons\\World Map\\icon_map_office.dds",
    "Interface\\Icons\\World Map\\icon_map_ruins_town.dds",
    "Interface\\Icons\\World Map\\icon_map_ruins_urban.dds",
    "Interface\\Icons\\World Map\\icon_map_ruins_sewer.dds",
    "Interface\\Icons\\World Map\\icon_map_metro.dds",
    "Interface\\Icons\\World Map\\icon_map_vault.dds",
];
pub const UNDISCOVERED_PICTURE: &str = "Interface\\Icons\\World Map\\icon_map_undiscovered.dds";

/// The world map's markers' size at a magnification (`0079c5a0`: the
/// magnification's share between `fWorldMapMinZoom` 0.75 and `MaxZoom` 5,
/// clamped, in a straight line from `fWorldMapMarkerMinSize` 20 to
/// `MaxSize` 50).
pub fn marker_size(magnification: f32) -> f32 {
    let k = zoom_share(magnification);
    20.0 + (50.0 - 20.0) * k
}

/// `fWorldMapMinZoom` and `fWorldMapMaxZoom` (`011d22f8`, `011d3d3c`).
pub const WORLD_MIN_ZOOM: f32 = 0.75;
pub const WORLD_MAX_ZOOM: f32 = 5.0;
/// One notch of the mouse wheel zooms by this (`0079c530`: `01056528`
/// 1.1), Page Up / Page Down (the pad's bumpers) by this (`00799790`
/// cases 0x0f / 0x10: `01018204` 1.2).
pub const WHEEL_ZOOM: f32 = 1.1;
pub const KEY_ZOOM: f32 = 1.2;

/// A magnification's share between the world map's least and most zoom,
/// clamped.
fn zoom_share(magnification: f32) -> f32 {
    ((magnification - WORLD_MIN_ZOOM) / (WORLD_MAX_ZOOM - WORLD_MIN_ZOOM)).clamp(0.0, 1.0)
}

/// The world map's `user0` .. `user2` (`0079c5a0`): the markers'
/// (`fWorldMapMarkerMinSize` 20 .. `MaxSize` 50), the quest markers'
/// (`fWorldQuestMarkerMinSize` 30 .. 60) and the player's arrow's
/// (`fWorldfPlayerCursorMinSize` 40 .. `fWorldPlayerCursorMaxSize` 70)
/// widths, in a straight line by the magnification's share.
fn set_marker_sizes(ui: &mut Ui, world: TileId, magnification: f32) {
    let k = zoom_share(magnification);
    ui.set_number(world, t::USER0, marker_size(magnification));
    ui.set_number(world, t::USER0 + 1, 30.0 + 30.0 * k);
    ui.set_number(world, t::USER0 + 2, 40.0 + 30.0 * k);
}

/// The DATA menu.
pub struct DataMenu {
    pub menu: TileId,
    pub tab: usize,
    pub tabline: Option<TileId>,
    pub tabs: Vec<TileId>,
    pub quests: ListBox,
    pub notes: ListBox,
    pub radio: ListBox,
    pub objectives: ListBox,
    world: Option<TileId>,
    cursor: Option<TileId>,
    data_rect: Option<TileId>,
    /// The world map's markers' tiles, and the one chosen.
    markers: Vec<TileId>,
    pub marker: Option<usize>,
    /// The player's own marker's tile on the map.
    markers_extra: Vec<TileId>,
    /// What the map and the lists were last made from (each made again
    /// only when its own part changes, so dragging, the pointer's row and
    /// scrolling stay).
    filled_map: Option<Option<super::WorldMapLine>>,
    filled_lists: Option<(
        Vec<QuestLine>,
        Vec<super::NoteLine>,
        Vec<super::StationLine>,
    )>,
    /// The last row the pointer was over (`011da400`).
    pub(crate) hovered: Option<usize>,
    /// The game's cursor is hidden (its alpha 0) while it's over the map:
    /// the highlight box follows it instead (`0079a130`).
    pub cursor_hidden: bool,
    /// The map's place when the button went down (the menu's +0x120 /
    /// +0x124, stored by `00798480`, `MapMenu::DoDownClick` (Xbox PDB),
    /// for a press on the world map, id 4; compared by `00796fd0` case
    /// 0x1a): a release after the map moved is a drag, not a click on a
    /// marker.
    pressed_at: Option<(f32, f32)>,
    /// The note shown and marked (`+0x90`), whose audio plays.
    pub playing: Option<u32>,
}

/// The list rows' `id`s the click and mouse-over handlers look for
/// (`00796fd0`, `00798cb0`): 0x17 a quest (its objectives shown, `_ItemType`
/// 4; clicked, made the active quest by `009529d0`), 0x18 a note (shown by
/// its kind, `007993d0`), 0x19 a radio station.
pub const QUEST_ROW_ID: i32 = 0x17;
pub const NOTE_ROW_ID: i32 = 0x18;
pub const RADIO_ROW_ID: i32 = 0x19;
/// The first tab button's `id` (0x20 Local Map .. 0x24 Radio, `00796b90`).
pub const FIRST_TAB_ID: i32 = 0x20;
/// The world map's picture (`MM_WorldMap_ParentImage`) and the local
/// map's (`MM_LocalMap_ParentImage`): `id` 4 and 2.
pub const WORLD_MAP_ID: i32 = 4;
pub const LOCAL_MAP_ID: i32 = 2;
/// A map marker's `id` (`MapMarkerTemplate`, `0079cdb0`): 26, compared as
/// `01074f60` (26.0) by `00799dc0`.
pub const MARKER_ID: i32 = 26;
/// The player's own marker's `id` (`0079f360`: 0x1c).
pub const CUSTOM_MARKER_ID: i32 = 0x1c;
/// A quest marker's `id` (`0079e0a0`: 0x1b).
pub const QUEST_MARKER_ID: i32 = 0x1b;
/// The part of the map clip window the cursor is "on the map" in
/// (`0079a130`: 0 .. `01074f68` 850 across, 0 .. `010301a8` 500
/// down).
pub const MAP_AREA: [f32; 2] = [850.0, 500.0];

impl DataMenu {
    /// Reads the menu, builds its tab line (`00796b90`: ids from 0x20,
    /// `sLocalMapTabText` .. `sCommsTabText`) and starts on the world map.
    pub fn load(
        ui: &mut Ui,
        read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ) -> Result<DataMenu, String> {
        let menu = super::load_menu(ui, super::DATA_FILE, read)?;
        let list = |ui: &mut Ui, id: i32| {
            let tile = by_id(ui, menu, id).unwrap_or(menu);
            ListBox::new(menu, tile, "MM_ListMarkerTemplate")
        };
        let quests = list(ui, 7);
        let notes = list(ui, 8);
        let radio = list(ui, 10);
        let objectives = list(ui, 15);
        let tabline_tile = by_id(ui, menu, 17);
        let labels: Vec<String> = [
            "sLocalMapTabText",
            "sWorldMapTabText",
            "sQuestsTabText",
            "sMiscTabText",
            "sCommsTabText",
        ]
        .iter()
        .map(|s| text(ui, s))
        .collect();
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let tabs = match tabline_tile {
            Some(tl) => tabline::build(ui, menu, tl, 0x20, &refs),
            None => Vec::new(),
        };
        let mut d = DataMenu {
            menu,
            tab: 1,
            tabline: tabline_tile,
            tabs,
            quests,
            notes,
            radio,
            objectives,
            world: by_id(ui, menu, 4),
            cursor: by_id(ui, menu, 5),
            data_rect: by_id(ui, menu, 13),
            markers: Vec::new(),
            marker: None,
            markers_extra: Vec::new(),
            filled_map: None,
            filled_lists: None,
            hovered: None,
            cursor_hidden: false,
            pressed_at: None,
            playing: None,
        };
        d.set_tab(ui, 1);
        Ok(d)
    }

    fn set_tab(&mut self, ui: &mut Ui, tab: usize) {
        self.tab = tab.min(4);
        if let Some(tl) = self.tabline {
            tabline::set_current(ui, tl, self.tab);
        }
        if let Some(rect) = self.data_rect {
            ui.set_number(
                rect,
                t::VISIBLE,
                if (2..=3).contains(&self.tab) {
                    1.0
                } else {
                    0.0
                },
            );
        }
    }

    /// The place and time, the map and its markers, the lists.
    pub fn fill(&mut self, ui: &mut Ui, input: &PipboyInput) {
        if let Some(tile) = by_id(ui, self.menu, 0) {
            ui.set_string(tile, t::STRING, &input.location);
        }
        if let Some(tile) = by_id(ui, self.menu, 1) {
            ui.set_string(tile, t::STRING, &input.date_time);
        }
        let mut changed = false;
        if self.filled_map.as_ref() != Some(&input.world_map) {
            self.fill_world_map(ui, input);
            self.filled_map = Some(input.world_map.clone());
            changed = true;
        }
        let lists = (
            input.quests.clone(),
            input.notes.clone(),
            input.stations.clone(),
        );
        if self.filled_lists.as_ref() != Some(&lists) {
            self.fill_quests(ui, &input.quests);
            let keep = self.notes.selected;
            self.notes.clear(ui);
            for n in &input.notes {
                if let Some(row) = self.notes.add(ui, Some(&n.name)) {
                    ui.set_number(row, t::ID, NOTE_ROW_ID as f32);
                }
            }
            if !input.notes.is_empty() {
                self.notes
                    .select(ui, Some(keep.unwrap_or(0).min(input.notes.len() - 1)));
            }
            // Translated from 0079bea0 (decompiled, FalloutNV.exe 1.4.0.525):
            // id 0x19, `_selected` the tuned station, `_TextAlpha` 255 in
            // range, else 128 (the rows come sorted).
            self.radio.clear(ui);
            let selected = trait_id(ui, "_selected");
            let alpha = trait_id(ui, "_TextAlpha");
            for s in &input.stations {
                if let Some(row) = self.radio.add(ui, Some(&s.name)) {
                    ui.set_number(row, t::ID, RADIO_ROW_ID as f32);
                    ui.set_number(row, selected, f32::from(u8::from(s.tuned)));
                    ui.set_number(row, alpha, if s.in_range { 255.0 } else { 128.0 });
                }
            }
            self.filled_lists = Some(lists);
            changed = true;
        }
        if changed {
            self.show_selected(ui, input);
        }
    }

    /// The world map (`0079cdb0`): the worldspace's picture with its
    /// usable size as the file size, a marker per map marker shown
    /// (`MapMarkerTemplate`: `_LocationName`, `id` 26, `_x`/`_y` its place,
    /// its picture by kind or the undiscovered one with `user0` 0, and
    /// `_MarkerIndex`), the player's arrow at the player's place.
    fn fill_world_map(&mut self, ui: &mut Ui, input: &PipboyInput) {
        let Some(world) = self.world else {
            return;
        };
        for m in self.markers.drain(..).chain(self.markers_extra.drain(..)) {
            ui.remove(m);
        }
        // The same picture again (only markers changed): the map stays
        // where it is and as zoomed.
        let same_picture = self
            .filled_map
            .as_ref()
            .and_then(|m| m.as_ref())
            .zip(input.world_map.as_ref())
            .is_some_and(|(a, b)| a.picture == b.picture && a.size == b.size);
        let Some(map) = &input.world_map else {
            ui.set_number(world, t::VISIBLE, 0.0);
            return;
        };
        ui.set_number(world, t::VISIBLE, 1.0);
        ui.set_string(world, t::FILENAME, &map.picture);
        ui.set_number(world, t::FILEWIDTH, map.size[0]);
        ui.set_number(world, t::FILEHEIGHT, map.size[1]);
        let mag = trait_id(ui, "_Magnification");
        let magnification = ui.number(world, mag).max(0.01);
        set_marker_sizes(ui, world, magnification);
        let (x_id, y_id) = (trait_id(ui, "_x"), trait_id(ui, "_y"));
        let name_id = trait_id(ui, "_LocationName");
        let index_id = trait_id(ui, "_MarkerIndex");
        for (i, m) in map.markers.iter().enumerate() {
            let Some(tile) = ui.instantiate(self.menu, world, "MapMarkerTemplate") else {
                continue;
            };
            ui.set_string(tile, name_id, &m.name);
            ui.set_number(tile, t::ID, 26.0);
            ui.set_number(tile, x_id, m.at[0]);
            ui.set_number(tile, y_id, m.at[1]);
            if m.travel {
                let picture = MARKER_PICTURES
                    .get(usize::from(m.kind))
                    .copied()
                    .unwrap_or("");
                ui.set_string(tile, t::FILENAME, picture);
            } else {
                ui.set_string(tile, t::FILENAME, UNDISCOVERED_PICTURE);
                ui.set_number(tile, t::USER0, 0.0);
            }
            ui.set_number(tile, index_id, i as f32);
            self.markers.push(tile);
        }
        // The player's own marker (`0079f360`, `MapMenu::CreatePlayerMarker`
        // (Xbox PDB)): a `WorldMapQuestMarkerTemplate` with the compass's
        // `glow_hud_compass_pc_marker.dds`, `id` 0x1c, at its place.
        if let Some(at) = map.custom {
            if let Some(tile) = ui.instantiate(self.menu, world, "WorldMapQuestMarkerTemplate") {
                ui.set_number(tile, x_id, at[0]);
                ui.set_number(tile, y_id, at[1]);
                ui.set_string(tile, t::FILENAME, "glow_hud_compass_pc_marker.dds");
                ui.set_number(tile, t::ID, CUSTOM_MARKER_ID as f32);
                self.markers_extra.push(tile);
            }
        }
        // The active quest's targets (`0079e0a0`, `MapMenu::AddQuestMarkers`
        // (Xbox PDB), with `bShowQuestMarkers`): a `WorldMapQuestMarker
        // Template` each, `glow_hud_compass_objective_marker.dds`, `id` 0x1b,
        // its brightness the map's.
        // Translated from 0079e0a0 (decompiled, FalloutNV.exe 1.4.0.525)
        for at in &map.quest {
            if let Some(tile) = ui.instantiate(self.menu, world, "WorldMapQuestMarkerTemplate") {
                ui.set_number(tile, x_id, at[0]);
                ui.set_number(tile, y_id, at[1]);
                ui.set_string(tile, t::FILENAME, "glow_hud_compass_objective_marker.dds");
                ui.set_number(tile, t::ID, QUEST_MARKER_ID as f32);
                ui.set_number(tile, t::BRIGHTNESS, 0.0);
                ui.add_action(
                    tile,
                    t::BRIGHTNESS,
                    crate::names::op::COPY,
                    crate::tile::Operand::Link {
                        tile: world,
                        trait_id: t::BRIGHTNESS,
                    },
                );
                self.markers_extra.push(tile);
            }
        }
        if let Some(cursor) = self.cursor {
            match map.player {
                Some((at, _heading)) => {
                    ui.set_number(cursor, x_id, at[0]);
                    ui.set_number(cursor, y_id, at[1]);
                    ui.set_number(cursor, t::VISIBLE, 1.0);
                }
                None => ui.set_number(cursor, t::VISIBLE, 0.0),
            }
        }
        // Centred on the player to start with (the map's `x` keeps adding
        // the mouse's drag and stays inside the window: its own operators).
        let centre = map.player.map(|(at, _)| at);
        self.marker = None;
        if let Some(at) = centre.filter(|_| !same_picture) {
            self.centre_on(ui, at);
        }
    }

    /// Moves the world map so a place on it sits in the window's middle
    /// (427.5, 250: where the highlight box is).
    fn centre_on(&mut self, ui: &mut Ui, at: [f32; 2]) {
        let Some(world) = self.world else {
            return;
        };
        let w = ui.number(world, t::WIDTH);
        let h = ui.number(world, t::HEIGHT);
        ui.set_base(world, t::X, 427.5 - at[0] * w);
        ui.set_base(world, t::Y, 250.0 - at[1] * h);
    }

    /// Zooms the world map in or out by `factor` about the point at the
    /// clip window's middle. Translated from 0079c5a0 (decompiled,
    /// FalloutNV.exe 1.4.0.525), `MapMenu::ZoomMap` (Xbox PDB): only on the
    /// world map's tab here (the local map, which the game zooms the same
    /// way between `fLocalMapMinZoom` and `MaxZoom`, isn't drawn); the
    /// magnification × or ÷ `factor`, clamped to `fWorldMapMinZoom` ..
    /// `MaxZoom`; the markers' sizes from it; the map's point under the
    /// window's middle (half the window less the map's `x`, `y`) scaled
    /// by the new width over the old and put back there (the map's `x`
    /// less its `dragdeltax`, as the file adds that); when the
    /// magnification changed, the scroll knob turns (`007f8610`, which
    /// plays `UIPipBoyScroll`).
    pub fn zoom(&mut self, ui: &mut Ui, zoom_in: bool, factor: f32) -> Vec<Action> {
        let mut out = Vec::new();
        let Some(world) = self.world.filter(|_| self.tab == 1) else {
            return out;
        };
        if ui.number(world, t::VISIBLE) == 0.0 {
            return out;
        }
        let Some(window) = ui.tiles[world].parent else {
            return out;
        };
        let mag_id = trait_id(ui, "_Magnification");
        let old = ui.number(world, mag_id);
        let by = if zoom_in { factor } else { 1.0 / factor };
        let new = (old * by).clamp(WORLD_MIN_ZOOM, WORLD_MAX_ZOOM);
        let (ww, wh) = (ui.number(window, t::WIDTH), ui.number(window, t::HEIGHT));
        let old_width = ui.number(world, t::WIDTH);
        let mut centre = (
            ww / 2.0 - ui.number(world, t::X),
            wh / 2.0 - ui.number(world, t::Y),
        );
        set_marker_sizes(ui, world, new);
        if old_width > 0.0 {
            let k = ui.number(world, t::FILEWIDTH) * new / old_width;
            centre = (centre.0 * k, centre.1 * k);
        }
        ui.set_number(world, mag_id, new);
        ui.refresh();
        let delta = (
            ui.number(world, crate::menu::drag::DELTA_X),
            ui.number(world, crate::menu::drag::DELTA_Y),
        );
        ui.set_base(world, t::X, ww / 2.0 - centre.0 - delta.0);
        ui.set_base(world, t::Y, wh / 2.0 - centre.1 - delta.1);
        ui.refresh();
        if new != old {
            out.push(Action::Sound("UIPipBoyScroll".into()));
            out.push(Action::ScrollKnob { down: zoom_in });
        }
        out
    }

    /// The mouse wheel over the menu: on the map tabs, a notch away from
    /// the player zooms in. Translated from 0079c530 (decompiled,
    /// FalloutNV.exe 1.4.0.525): the interface's wheel count (120 a notch)
    /// ÷ 120 above 0 zooms in by 1.1, else out.
    pub fn wheel(&mut self, ui: &mut Ui, delta: i32) -> Vec<Action> {
        if self.tab > 1 {
            return Vec::new();
        }
        self.zoom(ui, delta / 120 > 0, WHEEL_ZOOM)
    }

    /// The quests (`MM_ListMarkerTemplate`: the active one's square filled,
    /// `_selected`). (How the game marks finished quests in the list isn't
    /// traced: they're listed as the others.)
    fn fill_quests(&mut self, ui: &mut Ui, quests: &[QuestLine]) {
        let keep = self.quests.selected;
        self.quests.clear(ui);
        let selected = trait_id(ui, "_selected");
        for q in quests {
            if let Some(row) = self.quests.add(ui, Some(&q.name)) {
                ui.set_number(row, selected, if q.active { 1.0 } else { 0.0 });
                ui.set_number(row, t::ID, QUEST_ROW_ID as f32);
            }
        }
        if !quests.is_empty() {
            self.quests
                .select(ui, Some(keep.unwrap_or(0).min(quests.len() - 1)));
        }
    }

    /// What's chosen shows in the data rectangle: a quest's objectives
    /// (`_ItemType` 4, each a row, done ones' squares filled), a note's
    /// text (1); the world map's chosen marker's name in the highlight box.
    fn show_selected(&mut self, ui: &mut Ui, input: &PipboyInput) {
        self.show_marker_title(ui, input);
        let item_type = trait_id(ui, "_ItemType");
        let Some(rect) = self.data_rect else {
            return;
        };
        match self.tab {
            2 => {
                ui.set_number(rect, item_type, 4.0);
                self.objectives.clear(ui);
                let selected = trait_id(ui, "_selected");
                let show_empty = trait_id(ui, "_ShowEmptyMarker");
                if let Some(q) = self.quests.selected.and_then(|i| input.quests.get(i)) {
                    for (text, done) in &q.objectives {
                        if let Some(row) = self.objectives.add(ui, Some(text)) {
                            ui.set_number(row, show_empty, 1.0);
                            ui.set_number(row, selected, if *done { 1.0 } else { 0.0 });
                        }
                    }
                }
            }
            3 => {
                if let Some(n) = self
                    .notes
                    .selected
                    .and_then(|i| input.notes.get(i))
                    .cloned()
                {
                    self.show_note(ui, rect, &n, input);
                } else {
                    ui.set_number(rect, item_type, 0.0);
                }
            }
            _ => ui.set_number(rect, item_type, 0.0),
        }
    }

    /// A note in the data rectangle by its kind (`007993d0`): a sound or
    /// voice note's audio meter (`_ItemType` 3; "--:--:- remaining" and the
    /// meter at 0 when nothing plays), a text note's text (1; its scrollbar
    /// as many steps of `_ScrollDelta` as the text runs past the box, plus
    /// one, at the top), a picture note's picture (2, `XNAM`).
    // Translated from 007993d0 (decompiled, FalloutNV.exe 1.4.0.525)
    fn show_note(&mut self, ui: &mut Ui, rect: TileId, n: &super::NoteLine, input: &PipboyInput) {
        let item_type = trait_id(ui, "_ItemType");
        match n.kind {
            0 | 3 => {
                ui.set_number(rect, item_type, 3.0);
                if input.note_audio.is_none() {
                    let info = trait_id(ui, "_ItemStringInfo");
                    let s = format!("--:--:- {}", text(ui, "sRemaining"));
                    ui.set_string(rect, info, &s);
                    let done = trait_id(ui, "_AudioFileCompletion");
                    ui.set_number(rect, done, 0.0);
                }
            }
            2 => {
                if let Some(image) = ui.find_below(rect, "MM_DataImage") {
                    ui.set_string(image, t::FILENAME, n.image.as_deref().unwrap_or(""));
                }
                ui.set_number(rect, item_type, 2.0);
            }
            _ => {
                ui.set_number(rect, item_type, 1.0);
                let text_tile = ui.find_below(rect, "MM_DataText");
                if let Some(text_tile) = text_tile {
                    ui.set_string(text_tile, t::STRING, &n.text);
                }
                if let (Some(text_tile), Some(bar), Some(area)) = (
                    text_tile,
                    ui.find_below(rect, "MM_TextScrollbar"),
                    ui.find_below(rect, "MM_DataTextRect"),
                ) {
                    ui.refresh();
                    let delta_id = trait_id(ui, "_ScrollDelta");
                    let delta = ui.number(bar, delta_id).max(1.0);
                    let over =
                        (ui.number(text_tile, t::HEIGHT) - ui.number(area, t::HEIGHT)).max(0.0);
                    let items = trait_id(ui, "_number_of_items");
                    ui.set_number(bar, items, (over / delta).ceil() + 1.0);
                    let current = trait_id(ui, "_current_value");
                    ui.set_number(bar, current, 0.0);
                    let poke = trait_id(ui, "_SetInCode");
                    ui.set_number(bar, poke, 1.0);
                }
            }
        }
    }

    /// A note's audio playing, every frame (`0079a660`): the time since it
    /// began as "%02d:%02d:%01d remaining" (minutes, seconds, tenths) and
    /// the meter's share; when it has all played "00:00:0 remaining", the
    /// meter full, `user0` 0, the note no longer marked playing.
    // Translated from 0079a660 (decompiled, FalloutNV.exe 1.4.0.525)
    pub fn note_audio(&mut self, ui: &mut Ui, audio: Option<super::NoteAudio>) {
        let (Some(a), Some(rect)) = (audio, self.data_rect) else {
            return;
        };
        if self.playing != Some(a.note) {
            return;
        }
        let info = trait_id(ui, "_ItemStringInfo");
        let completion = trait_id(ui, "_AudioFileCompletion");
        let remaining = text(ui, "sRemaining");
        if a.done {
            ui.set_string(rect, info, &format!("00:00:0 {remaining}"));
            ui.set_number(rect, completion, 1.0);
            ui.set_number(rect, t::USER0, 0.0);
            self.mark_playing(ui, None);
            return;
        }
        let ms = a.elapsed_ms.min(a.total_ms).max(0.0) as i32;
        let s = format!(
            "{:02}:{:02}:{:01} {remaining}",
            ms / 60000,
            (ms % 60000) / 1000,
            (ms % 1000) / 100
        );
        ui.set_string(rect, info, &s);
        if a.total_ms > 0.0 {
            ui.set_number(rect, completion, a.elapsed_ms.min(a.total_ms) / a.total_ms);
        }
    }

    /// Marks a note's row as the one playing (`_selected`, the playing
    /// mark `011da348`), the others not.
    fn mark_playing(&mut self, ui: &mut Ui, note: Option<u32>) {
        let selected = trait_id(ui, "_selected");
        let notes = self
            .filled_lists
            .as_ref()
            .map(|l| l.1.clone())
            .unwrap_or_default();
        for (row, n) in self.notes.rows.clone().into_iter().zip(&notes) {
            ui.set_number(row, selected, if Some(n.form) == note { 1.0 } else { 0.0 });
        }
        self.playing = note;
    }

    /// The chosen marker's name in the highlight box (its `_Title`).
    fn show_marker_title(&mut self, ui: &mut Ui, input: &PipboyInput) {
        if let Some(highlight) = by_id(ui, self.menu, 6) {
            let title = trait_id(ui, "_Title");
            let name = self
                .marker
                .and_then(|i| input.world_map.as_ref()?.markers.get(i))
                .map(|m| m.name.clone())
                .unwrap_or_default();
            // `00799dc0` leaves the title as it was for "Companion".
            if !name.eq_ignore_ascii_case("Companion") {
                ui.set_string(highlight, title, &name);
            }
        }
    }

    /// Shows a tab (0 Local Map .. 4 Radio).
    pub fn show_tab(&mut self, ui: &mut Ui, tab: usize, input: &PipboyInput) {
        self.set_tab(ui, tab);
        self.show_selected(ui, input);
    }

    /// A key (`00799790`): left and right change tab, wrapping round the
    /// five; up and down choose in the Quests, Misc and Radio lists; the A
    /// button on the world map presses the marker the highlight box found
    /// under the pointer ([`DataMenu::pointer_moved`]; the pad's stick isn't
    /// here), on a quest makes it the active one.
    pub fn key(&mut self, ui: &mut Ui, key: Key, input: &PipboyInput) -> Vec<Action> {
        let mut out = Vec::new();
        match key {
            Key::Left | Key::Right => {
                let tab = if key == Key::Right {
                    (self.tab + 1) % 5
                } else {
                    (self.tab + 4) % 5
                };
                self.set_tab(ui, tab);
                self.show_selected(ui, input);
                out.push(Action::Sound("UIPipBoyTab".into()));
            }
            Key::Up | Key::Down => {
                let by = if key == Key::Down { 1 } else { -1 };
                let list = match self.tab {
                    2 => Some(&mut self.quests),
                    3 => Some(&mut self.notes),
                    4 => Some(&mut self.radio),
                    _ => None,
                };
                if let Some(list) = list {
                    let before = list.selected;
                    list.step(ui, by);
                    if list.selected != before {
                        out.push(Action::Sound("UIPipBoyScroll".into()));
                    }
                }
                self.show_selected(ui, input);
            }
            // The bumpers (Page Up / Page Down) zoom the maps, Page Down
            // in (`00799790` cases 0x0f / 0x10).
            Key::PageUp | Key::PageDown => {
                if self.tab <= 1 {
                    out.extend(self.zoom(ui, key == Key::PageDown, KEY_ZOOM));
                }
            }
            Key::Activate => match self.tab {
                1 => {
                    if let Some(m) = self
                        .marker
                        .and_then(|i| input.world_map.as_ref()?.markers.get(i))
                    {
                        out.push(Action::Travel(m.form));
                    }
                }
                2 => {
                    if let Some(q) = self.quests.selected.and_then(|i| input.quests.get(i)) {
                        out.push(Action::ActiveQuest(q.form));
                    }
                }
                3 => out.extend(self.click(ui, NOTE_ROW_ID, None, input)),
                _ => {}
            },
            _ => {}
        }
        out
    }

    /// The list a row tile is in (quests, notes, radio), by its `id`.
    fn row_list(&mut self, id: i32) -> Option<&mut ListBox> {
        match id {
            QUEST_ROW_ID => Some(&mut self.quests),
            NOTE_ROW_ID => Some(&mut self.notes),
            RADIO_ROW_ID => Some(&mut self.radio),
            _ => None,
        }
    }

    /// A tile clicked (`00796fd0`, slot 0x0c): a tab button (0x20 .. 0x24)
    /// shows its tab; a quest row (0x17) makes that quest the active one
    /// unless it's finished (`0059e400`, `009529d0`); the world map's
    /// picture (4) presses the marker the highlight box has found (case 4
    /// → 0x1a), when the map wasn't dragged since the button went down and
    /// the marker can be travelled to (the caller then asks "%s %s?",
    /// `00703e80` with `00798710`). (Notes' audio, the radio playing and
    /// challenges are not here yet; the player's marker, 0x0c, comes from
    /// the right button: [`DataMenu::right_pressed`].)
    pub fn click(
        &mut self,
        ui: &mut Ui,
        id: i32,
        tile: Option<TileId>,
        input: &PipboyInput,
    ) -> Vec<Action> {
        let mut out = Vec::new();
        match id {
            FIRST_TAB_ID..=0x24 => {
                self.set_tab(ui, (id - FIRST_TAB_ID) as usize);
                self.show_selected(ui, input);
                out.push(Action::Sound("UIPipBoyTab".into()));
            }
            QUEST_ROW_ID => {
                let index = tile.and_then(|t| self.quests.index_of(t));
                if let Some(q) = index.and_then(|i| input.quests.get(i)) {
                    if !q.completed {
                        out.push(Action::ActiveQuest(q.form));
                    }
                }
            }
            // A note (`00796fd0` case 0x18): clicked again it's let go (an
            // audio one stops); else shown and marked, `user0` 1, and a
            // sound or voice note starts playing (`UIPipBoyHolotapeStart`).
            // Translated from 00796fd0 (decompiled, FalloutNV.exe 1.4.0.525)
            NOTE_ROW_ID => {
                let index = tile
                    .and_then(|t| self.notes.index_of(t))
                    .or(self.notes.selected);
                let Some(n) = index.and_then(|i| input.notes.get(i)).cloned() else {
                    return out;
                };
                let audio = matches!(n.kind, 0 | 3);
                let same = self.playing == Some(n.form);
                let was_playing = input.note_audio.is_some();
                if was_playing {
                    out.push(Action::StopNote);
                }
                let rect = self.data_rect;
                if !same || (audio && !was_playing) {
                    if let Some(i) = index {
                        self.notes.choose(ui, Some(i));
                    }
                    if let Some(rect) = rect {
                        self.show_note(ui, rect, &n, input);
                        ui.set_number(rect, t::USER0, 1.0);
                    }
                    self.mark_playing(ui, Some(n.form));
                } else {
                    if let Some(rect) = rect {
                        ui.set_number(rect, t::USER0, 0.0);
                    }
                    self.mark_playing(ui, None);
                }
                if audio && (!same || !was_playing) && self.playing == Some(n.form) {
                    out.push(Action::PlayNote(n.form));
                    out.push(Action::Sound("UIPipBoyHolotapeStart".into()));
                }
            }
            // A station's row (`00796fd0` case 0x19): only one in range
            // reacts (its `_TextAlpha` 255).
            RADIO_ROW_ID => {
                let index = tile.and_then(|t| self.radio.index_of(t));
                if let Some(s) = index.and_then(|i| input.stations.get(i)) {
                    if s.in_range {
                        out.push(Action::Radio(s.reference));
                    }
                }
            }
            WORLD_MAP_ID if self.tab == 1 => {
                let unmoved = match (self.pressed_at, self.world) {
                    (Some(at), Some(w)) => at == (ui.number(w, t::X), ui.number(w, t::Y)),
                    _ => false,
                };
                if let Some(m) = self
                    .marker
                    .filter(|_| unmoved)
                    .and_then(|i| input.world_map.as_ref()?.markers.get(i))
                {
                    if m.travel {
                        out.push(Action::Travel(m.form));
                    }
                }
            }
            _ => {}
        }
        out
    }

    /// The right button went down (`0079a130` on the map tabs, the pointer
    /// inside the map's area, the Pip-Boy control up: click 0x0c): the
    /// player's marker is asked for at the point under the pointer. Case
    /// 0x0c of 00796fd0 (decompiled, FalloutNV.exe 1.4.0.525), the mouse's
    /// branch: the pointer less the map's place on the screen, both
    /// divided by the map's width (`0053d280`), on the world map's tab
    /// (the local map isn't drawn here).
    pub fn right_pressed(&mut self, ui: &mut Ui, at: Option<[f32; 2]>) -> Vec<Action> {
        let mut out = Vec::new();
        let (Some(at), Some(world)) = (at, self.world) else {
            return out;
        };
        if self.tab != 1 || !self.cursor_hidden || ui.number(world, t::VISIBLE) == 0.0 {
            return out;
        }
        let (mx, my) = ui.screen_position(world);
        let width = ui.number(world, t::WIDTH);
        if width > 0.0 {
            out.push(Action::PlaceMarker([
                (at[0] - mx) / width,
                (at[1] - my) / width,
            ]));
        }
        out
    }

    /// The button went down over the menu: where the map is then.
    pub fn pressed(&mut self, ui: &mut Ui) {
        self.pressed_at = self.world.map(|w| (ui.number(w, t::X), ui.number(w, t::Y)));
    }

    /// The pointer onto a tile (`00798cb0`, slot 0x10): a quest, note or
    /// radio row becomes the list's choice and shows what it holds (a
    /// quest's objectives, a note's text); the knob clicks (`007f8610`,
    /// `UIPipBoyScroll`) when its `listindex` differs from the last row the
    /// pointer was over.
    pub fn mouseover(
        &mut self,
        ui: &mut Ui,
        id: i32,
        tile: TileId,
        input: &PipboyInput,
    ) -> Vec<Action> {
        let mut out = Vec::new();
        let Some(list) = self.row_list(id) else {
            return out;
        };
        let Some(index) = list.index_of(tile) else {
            return out;
        };
        list.choose(ui, Some(index));
        if self.hovered != Some(index) {
            self.hovered = Some(index);
            out.push(Action::Sound("UIPipBoyScroll".into()));
        }
        self.show_selected(ui, input);
        out
    }

    /// The pointer over the menu (`0079a130`, every frame on the Local Map
    /// and World Map tabs): the highlight box is centred on it (`x`, `y`
    /// in its clip window); inside the map's area (0 .. 850 × 0 .. 500 of
    /// that window) the game's cursor is hidden; then the marker nearest
    /// the box's centre within half its height is found (`00799dc0`): its
    /// name becomes the box's `_Title` (not for "Companion") and
    /// `UIPipBoyHighlight` plays when it changes. (The game measures the
    /// markers in the map's own frame from their `_x`/`_y`; here their
    /// centres on the screen, the same points as the file lays them out.)
    pub fn pointer_moved(
        &mut self,
        ui: &mut Ui,
        at: Option<[f32; 2]>,
        input: &PipboyInput,
    ) -> Vec<Action> {
        let mut out = Vec::new();
        self.cursor_hidden = false;
        if self.tab > 1 {
            return out;
        }
        let Some(at) = at else {
            return out;
        };
        let Some(bx) = by_id(ui, self.menu, 6) else {
            return out;
        };
        let Some(frame) = ui.tiles[bx].parent else {
            return out;
        };
        let (fx, fy) = ui.screen_position(frame);
        let (lx, ly) = (at[0] - fx, at[1] - fy);
        self.cursor_hidden = (0.0..=MAP_AREA[0]).contains(&lx) && (0.0..=MAP_AREA[1]).contains(&ly);
        let (w, h) = (ui.number(bx, t::WIDTH), ui.number(bx, t::HEIGHT));
        ui.set_number(bx, t::X, lx - w / 2.0);
        ui.set_number(bx, t::Y, ly - h / 2.0);
        if self.tab != 1 {
            return out;
        }
        let (bsx, bsy) = ui.screen_position(bx);
        let centre = (bsx + w / 2.0, bsy + h / 2.0);
        let index_id = trait_id(ui, "_MarkerIndex");
        let mut best = f32::MAX;
        let mut found = None;
        for &m in &self.markers.clone() {
            let (mx, my) = ui.screen_position(m);
            let (mw, mh) = (ui.number(m, t::WIDTH), ui.number(m, t::HEIGHT));
            let d =
                ((mx + mw / 2.0 - centre.0).powi(2) + (my + mh / 2.0 - centre.1).powi(2)).sqrt();
            if d < best && d < h / 2.0 {
                best = d;
                found = Some(ui.number(m, index_id) as usize);
            }
        }
        if found != self.marker {
            self.marker = found;
            if found.is_some() {
                out.push(Action::Sound("UIPipBoyHighlight".into()));
            }
            self.show_selected(ui, input);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipboy::{MarkerLine, WorldMapLine};

    /// `map_menu.xml` cut down to the tiles the code finds by `id` and
    /// name.
    const MENU: &str = r#"<menu name="MapMenu"><locus>&true;</locus>
      <text name="location"><id>0</id></text><text name="time"><id>1</id></text>
      <image name="world"><id>4</id><width>855</width><height>500</height><_Magnification>1</_Magnification></image>
      <image name="cursor"><id>5</id></image>
      <rect name="highlight"><id>6</id></rect>
      <hotrect name="quests"><id>7</id><x>0</x><y>100</y><width>300</width><height>400</height>
        <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
        <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
      </hotrect>
      <hotrect name="notes"><id>8</id><x>0</x><y>100</y><width>300</width><height>400</height>
        <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
        <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
      </hotrect>
      <rect name="data"><id>13</id>
        <text name="MM_DataText"></text><image name="MM_DataImage"></image>
        <hotrect name="objectives"><id>15</id><x>0</x><y>0</y><width>300</width><height>400</height>
          <_scroll_delta>0</_scroll_delta><_highlight_y>-1</_highlight_y><_selected_height>0</_selected_height>
          <image name="lb_scrollbar"><_number_of_items>1</_number_of_items><_current_value>0</_current_value></image>
        </hotrect>
      </rect>
      <template name="MM_ListMarkerTemplate"><hotrect name="row"><height>30</height></hotrect></template>
      <template name="MapMarkerTemplate"><image name="marker"><user0>1</user0></image></template>
    </menu>"#;

    fn input() -> PipboyInput {
        let marker = |form: u32, name: &str, travel: bool| MarkerLine {
            form,
            name: name.into(),
            at: [0.25, 0.75],
            kind: 2,
            travel,
        };
        PipboyInput {
            location: "Goodsprings".into(),
            world_map: Some(WorldMapLine {
                picture: "interface\\worldmap\\test.dds".into(),
                size: [1000.0, 800.0],
                markers: vec![
                    marker(0x900, "Unknown Place", false),
                    marker(0x901, "Goodsprings", true),
                ],
                player: Some(([0.5, 0.5], 90.0)),
                corners: [[0.0, 1.0], [1.0, 0.0]],
                custom: None,
                quest: Vec::new(),
            }),
            quests: vec![QuestLine {
                form: 0x700,
                name: "Back in the Saddle".into(),
                completed: false,
                active: false,
                objectives: vec![("Meet Sunny".into(), true), ("Shoot bottles".into(), false)],
            }],
            ..PipboyInput::default()
        }
    }

    fn load(ui: &mut Ui) -> DataMenu {
        let mut read = |p: &str| (p == crate::pipboy::DATA_FILE).then(|| MENU.as_bytes().to_vec());
        DataMenu::load(ui, &mut read).unwrap()
    }

    /// `00796fd0` case 0x18, `007993d0`, `0079a660`: a text note shows its
    /// text, a picture note its picture; a voice note clicked plays
    /// (`UIPipBoyHolotapeStart`), marked and `user0` 1, its meter at
    /// "--:--:- remaining" until it plays, then the time since it began,
    /// "00:00:0 remaining" and full when done; clicked while playing it
    /// stops.
    #[test]
    fn notes_show_by_kind_and_play() {
        let mut ui = crate::pipboy::tests::ui();
        let mut d = load(&mut ui);
        let mut input = input();
        let note = |form: u32, name: &str, kind: u8| crate::pipboy::NoteLine {
            form,
            name: name.into(),
            text: "text".into(),
            kind,
            image: (kind == 2).then(|| "Architecture\\Urban\\MetroMap.dds".into()),
        };
        input.notes = vec![
            note(0x301, "Audio Log", 3),
            note(0x302, "Bunker", 1),
            note(0x303, "Metro Map", 2),
        ];
        d.fill(&mut ui, &input);
        d.show_tab(&mut ui, 3, &input);
        ui.refresh();
        let rect = by_id(&ui, d.menu, 13).unwrap();
        let item_type = ui.names.lookup("_ItemType").unwrap();
        let info = ui.names.lookup("_ItemStringInfo").unwrap();
        // The first, the audio log, chosen: its meter.
        assert_eq!(ui.number(rect, item_type), 3.0);
        assert_eq!(ui.string(rect, info).unwrap(), "--:--:- remaining");
        let out = d.click(&mut ui, NOTE_ROW_ID, Some(d.notes.rows[0]), &input);
        assert_eq!(
            out,
            [
                Action::PlayNote(0x301),
                Action::Sound("UIPipBoyHolotapeStart".into())
            ]
        );
        assert_eq!(ui.number(rect, t::USER0), 1.0);
        let selected = ui.names.lookup("_selected").unwrap();
        assert_eq!(ui.number(d.notes.rows[0], selected), 1.0);
        let audio = |elapsed_ms: f32, done: bool| crate::pipboy::NoteAudio {
            note: 0x301,
            elapsed_ms,
            total_ms: 90_000.0,
            done,
        };
        d.note_audio(&mut ui, Some(audio(65_432.0, false)));
        assert_eq!(ui.string(rect, info).unwrap(), "01:05:4 remaining");
        let completion = ui.names.lookup("_AudioFileCompletion").unwrap();
        assert!((ui.number(rect, completion) - 65_432.0 / 90_000.0).abs() < 1e-5);
        // Clicked again while it plays: it stops.
        input.note_audio = Some(audio(70_000.0, false));
        let out = d.click(&mut ui, NOTE_ROW_ID, Some(d.notes.rows[0]), &input);
        assert_eq!(out, [Action::StopNote]);
        assert_eq!(d.playing, None);
        // Played again to its end.
        input.note_audio = None;
        d.click(&mut ui, NOTE_ROW_ID, Some(d.notes.rows[0]), &input);
        d.note_audio(&mut ui, Some(audio(90_000.0, true)));
        assert_eq!(ui.string(rect, info).unwrap(), "00:00:0 remaining");
        assert_eq!(ui.number(rect, completion), 1.0);
        assert_eq!(ui.number(rect, t::USER0), 0.0);
        // The text note and the picture.
        let out = d.click(&mut ui, NOTE_ROW_ID, Some(d.notes.rows[1]), &input);
        assert!(out.is_empty());
        assert_eq!(ui.number(rect, item_type), 1.0);
        let text = ui.find_below(rect, "MM_DataText").unwrap();
        assert_eq!(ui.string(text, t::STRING).unwrap(), "text");
        d.click(&mut ui, NOTE_ROW_ID, Some(d.notes.rows[2]), &input);
        assert_eq!(ui.number(rect, item_type), 2.0);
        let image = ui.find_below(rect, "MM_DataImage").unwrap();
        assert_eq!(
            ui.string(image, t::FILENAME).unwrap(),
            "Architecture\\Urban\\MetroMap.dds"
        );
    }

    #[test]
    fn the_world_map_gets_its_picture_markers_and_the_player() {
        let mut ui = crate::pipboy::tests::ui();
        let mut d = load(&mut ui);
        let input = input();
        d.fill(&mut ui, &input);
        assert_eq!(d.tab, 1);
        let world = by_id(&ui, d.menu, 4).unwrap();
        assert_eq!(ui.number(world, t::FILEWIDTH), 1000.0);
        assert_eq!(ui.number(world, t::FILEHEIGHT), 800.0);
        // Markers' size at magnification 1: 20 + 30 × 0.25 / 4.25.
        assert!((ui.number(world, t::USER0) - (20.0 + 30.0 * 0.25 / 4.25)).abs() < 1e-4);
        assert_eq!(d.markers.len(), 2);
        let (x, y) = (
            ui.names.lookup("_x").unwrap(),
            ui.names.lookup("_y").unwrap(),
        );
        let found = d.markers[1];
        assert_eq!(ui.string(found, t::FILENAME).unwrap(), MARKER_PICTURES[2]);
        assert_eq!((ui.number(found, x), ui.number(found, y)), (0.25, 0.75));
        let unknown = d.markers[0];
        assert_eq!(
            ui.string(unknown, t::FILENAME).unwrap(),
            UNDISCOVERED_PICTURE
        );
        assert_eq!(ui.number(unknown, t::USER0), 0.0);
        let cursor = by_id(&ui, d.menu, 5).unwrap();
        assert_eq!(ui.number(cursor, x), 0.5);
        // Nothing is under a cursor: the A button presses no marker.
        assert!(d.key(&mut ui, Key::Activate, &input).is_empty());
        // Left from the world map: the local map; again: round to Radio.
        d.key(&mut ui, Key::Left, &input);
        assert_eq!(d.tab, 0);
        d.key(&mut ui, Key::Left, &input);
        assert_eq!(d.tab, 4);
        let location = by_id(&ui, d.menu, 0).unwrap();
        assert_eq!(ui.string(location, t::STRING).unwrap(), "Goodsprings");
    }

    /// `0079c5a0` / `0079c530` / `00799790`: zooming keeps the map's point
    /// under the window's middle there, clamps the magnification and sizes
    /// the markers by it; the wheel zooms by 1.1, Page Down in by 1.2.
    #[test]
    fn zooming_the_world_map_keeps_its_middle() {
        const ZOOM_MENU: &str = r#"<menu name="MapMenu"><locus>&true;</locus>
          <hotrect name="clip"><y>50</y><width>855</width><height>500</height><locus>&true;</locus>
            <hotrect name="world"><id>4</id><locus>&true;</locus><_Magnification>1</_Magnification>
              <width><copy src="me()" trait="filewidth"/><mul src="me()" trait="_Magnification"/></width>
              <height><copy src="me()" trait="fileheight"/><mul src="me()" trait="_Magnification"/></height>
              <x><add src="me()" trait="dragdeltax"/></x><y><add src="me()" trait="dragdeltay"/></y>
            </hotrect>
          </hotrect>
          <template name="MapMarkerTemplate"><image name="marker"><user0>1</user0></image></template>
        </menu>"#;
        let mut ui = crate::pipboy::tests::ui();
        let mut read =
            |p: &str| (p == crate::pipboy::DATA_FILE).then(|| ZOOM_MENU.as_bytes().to_vec());
        let mut d = DataMenu::load(&mut ui, &mut read).unwrap();
        d.fill(&mut ui, &input());
        let world = by_id(&ui, d.menu, 4).unwrap();
        let middle = |ui: &mut Ui| {
            let (x, y) = (ui.number(world, t::X), ui.number(world, t::Y));
            let (w, h) = (ui.number(world, t::WIDTH), ui.number(world, t::HEIGHT));
            ((427.5 - x) / w, (250.0 - y) / h)
        };
        assert_eq!(middle(&mut ui), (0.5, 0.5));
        let out = d.key(&mut ui, Key::PageDown, &input());
        assert_eq!(
            out,
            [
                Action::Sound("UIPipBoyScroll".into()),
                Action::ScrollKnob { down: true }
            ]
        );
        let mag = ui.names.lookup("_Magnification").unwrap();
        assert!((ui.number(world, mag) - 1.2).abs() < 1e-6);
        assert_eq!(ui.number(world, t::WIDTH), 1200.0);
        let (mx, my) = middle(&mut ui);
        assert!((mx - 0.5).abs() < 1e-5 && (my - 0.5).abs() < 1e-5);
        assert!((ui.number(world, t::USER0) - marker_size(1.2)).abs() < 1e-5);
        // A notch towards the player: out by 1.1.
        d.wheel(&mut ui, -120);
        assert!((ui.number(world, mag) - 1.2 / 1.1).abs() < 1e-5);
        // No further out than `fWorldMapMinZoom`, and no knob then.
        for _ in 0..10 {
            d.key(&mut ui, Key::PageUp, &input());
        }
        assert_eq!(ui.number(world, mag), WORLD_MIN_ZOOM);
        assert!(d.key(&mut ui, Key::PageUp, &input()).is_empty());
        // Not on the quests' tab.
        d.show_tab(&mut ui, 2, &input());
        assert!(d.wheel(&mut ui, 120).is_empty());
    }

    #[test]
    fn quests_list_their_objectives_and_can_be_made_active() {
        let mut ui = crate::pipboy::tests::ui();
        let mut d = load(&mut ui);
        let input = input();
        d.fill(&mut ui, &input);
        d.show_tab(&mut ui, 2, &input);
        let rect = by_id(&ui, d.menu, 13).unwrap();
        assert_eq!(ui.number(rect, t::VISIBLE), 1.0);
        let item_type = ui.names.lookup("_ItemType").unwrap();
        assert_eq!(ui.number(rect, item_type), 4.0);
        assert_eq!(d.quests.rows.len(), 1);
        assert_eq!(d.objectives.rows.len(), 2);
        // Done objectives' squares filled.
        let selected = ui.names.lookup("_selected").unwrap();
        let done: Vec<f32> = d
            .objectives
            .rows
            .clone()
            .into_iter()
            .map(|r| ui.number(r, selected))
            .collect();
        assert_eq!(done, [1.0, 0.0]);
        assert_eq!(
            d.key(&mut ui, Key::Activate, &input),
            [Action::ActiveQuest(0x700)]
        );
        // The world map's tab hides the data rectangle.
        d.key(&mut ui, Key::Left, &input);
        assert_eq!(ui.number(rect, t::VISIBLE), 0.0);
    }
}
