//! The tutorial message box (`menus\tutorial_menu.xml`, class
//! `TutorialMenu` 1059, vtable `0106ff84` in FalloutNV.exe): a help
//! message (`world::tutorial`) shown once, over the menu it's about. Read
//! from the code (names from the Xbox 360 prototype's PDB):
//!
//! * opening (`TutorialMenu::Create`, `007e8890`, with one message, as
//!   every caller in the game opens it): a tutorial menu already open is
//!   closed; the menu file loaded; `TM_VDSG_text` (id 7) gets
//!   `sVDSGManual`; Next (1) and Previous (2) are hidden; Close (3) gets
//!   `sCloseButton`; the page number (5) is hidden (its place taken by the
//!   file's `TM_fill_line`); then the message (`SetTitleAndText`,
//!   `007e9060`): the title (4) its `FULL` name, the text (0) its `DESC`
//!   (an HTML text when it starts with `<`), the scrollbar (6) back to the
//!   top (`_current_value` 0, its operators kept); then Next and Previous
//!   are made clickable or not by the page (`UpdatePrevNext`, `007e9010`:
//!   neither, with one message); and the menu is shown;
//! * a click (`007e8db0`): Next (1) and Previous (2) turn the help
//!   manual's pages (the start menu's Help, not opened here), Close (3)
//!   closes (`007e8d60`); the file's `_PCButton_E` is Close;
//! * the cancel code (10, `007e8e80`) closes it when it's the menu on top.

use crate::menu::{self, MenuCode};
use crate::names::t;
use crate::tile::{TileId, Ui};

/// The menu's file.
pub const FILE: &str = "menus\\tutorial_menu.xml";
/// Its class number (`&TutorialMenu;`).
pub const CLASS: i32 = 1059;
/// The code that closes it (`007e8e80`).
pub const CLOSE_CODE: i32 = 10;

/// The tiles by id (`007209e0`, ids 0–7).
pub mod tile {
    pub const TEXT: usize = 0;
    pub const NEXT: usize = 1;
    pub const PREVIOUS: usize = 2;
    pub const CLOSE: usize = 3;
    pub const TITLE: usize = 4;
    pub const PAGE: usize = 5;
    pub const SCROLLBAR: usize = 6;
    pub const VDSG_TEXT: usize = 7;
}

/// A help message: its record's `FULL` and `DESC`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Message {
    pub title: String,
    pub text: String,
}

/// The tutorial menu.
#[derive(Debug)]
pub struct TutorialMenu {
    pub menu: TileId,
    /// `pTiles` (+0x28).
    pub tiles: [Option<TileId>; 8],
    /// The page shown and how many there are (`iCurrentPage` +0x48,
    /// `iNumPages` +0x4c): 0 and 0 with one message.
    pub page: u32,
    pub pages: u32,
    /// The tutorial id it shows, for the caller.
    pub id: u8,
    pub closed: bool,
}

impl TutorialMenu {
    /// A tutorial menu for a menu tile loaded with [`menu::load`].
    pub fn new(menu: TileId, id: u8) -> TutorialMenu {
        TutorialMenu {
            menu,
            tiles: [None; 8],
            page: 0,
            pages: 0,
            id,
            closed: false,
        }
    }

    fn tile(&self, i: usize) -> Option<TileId> {
        self.tiles.get(i).copied().flatten()
    }

    /// Opens it with one message (`007e8890(message, 0)`).
    pub fn open(&mut self, ui: &mut Ui, message: &Message) {
        let text = |ui: &Ui, name: &str| ui.setting_text(name).unwrap_or_default();
        if let Some(vdsg) = self.tile(tile::VDSG_TEXT) {
            let s = text(ui, "sVDSGManual");
            ui.set_string(vdsg, t::STRING, &s);
        }
        for i in [tile::NEXT, tile::PREVIOUS] {
            if let Some(b) = self.tile(i) {
                ui.set_number(b, t::VISIBLE, 0.0);
            }
        }
        if let Some(close) = self.tile(tile::CLOSE) {
            let s = text(ui, "sCloseButton");
            ui.set_string(close, t::STRING, &s);
        }
        // No help manual: no pages.
        self.page = 0;
        self.pages = 0;
        if let Some(page) = self.tile(tile::PAGE) {
            ui.set_number(page, t::VISIBLE, 0.0);
        }
        self.set_title_and_text(ui, message);
        self.update_prev_next(ui);
        ui.set_number(self.menu, menu::LEAVE_STACK, 0.0);
        ui.set_number(self.menu, t::VISIBLE, 1.0);
        ui.refresh();
    }

    /// `SetTitleAndText` (`007e9060`).
    fn set_title_and_text(&mut self, ui: &mut Ui, message: &Message) {
        if let Some(title) = self.tile(tile::TITLE) {
            ui.set_string(title, t::STRING, &message.title);
        }
        if let Some(text) = self.tile(tile::TEXT) {
            let html = message.text.starts_with('<');
            ui.set_number(text, t::ISHTML, f32::from(html));
            ui.set_string(text, t::STRING, &message.text);
        }
        if let Some(bar) = self.tile(tile::SCROLLBAR) {
            let current = ui.names.lookup_or_add("_current_value").unwrap_or(0);
            ui.set_base(bar, current, 0.0);
        }
    }

    /// `UpdatePrevNext` (`007e9010`): Previous clickable past the first
    /// page, Next before the last.
    fn update_prev_next(&mut self, ui: &mut Ui) {
        if let Some(prev) = self.tile(tile::PREVIOUS) {
            ui.set_number(prev, t::TARGET, f32::from(self.page != 0));
        }
        if let Some(next) = self.tile(tile::NEXT) {
            ui.set_number(next, t::TARGET, f32::from(self.page + 1 < self.pages));
        }
    }

    /// Closes it (`007e8d60`).
    pub fn close(&mut self, ui: &mut Ui) {
        if self.closed {
            return;
        }
        ui.set_number(self.menu, menu::LEAVE_STACK, 1.0);
        self.closed = true;
    }
}

impl MenuCode for TutorialMenu {
    fn class(&self) -> i32 {
        CLASS
    }

    fn set_tile(&mut self, id: i32, tile: TileId) {
        if let Some(slot) = usize::try_from(id).ok().and_then(|i| self.tiles.get_mut(i)) {
            *slot = Some(tile);
        }
    }

    /// `007e8db0`: the help manual's pages aren't opened here, so Next and
    /// Previous have nothing to turn.
    fn click(&mut self, ui: &mut Ui, id: i32, _tile: Option<TileId>, _now: f64) {
        if id == tile::CLOSE as i32 {
            self.close(ui);
        }
    }

    /// `007e8e80`: the cancel code closes it.
    fn special_key(&mut self, ui: &mut Ui, code: i32, _now: f64) -> bool {
        if code == CLOSE_CODE {
            self.close(ui);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menus::test_support;

    /// The parts of `tutorial_menu.xml` the code reads.
    const XML: &str = "<menu name=\"TutorialMenu\"><class>&TutorialMenu;</class>
        <_PCButton_E> TM_close_button </_PCButton_E>
        <rect name=\"TM_depth_rect\">
          <text name=\"TM_title\"><id>4</id><font>6</font><string> Title Text </string></text>
          <text name=\"TM_page\"><id>5</id><font>6</font><string> ??/?? </string></text>
          <hotrect name=\"TM_scrollbar\"><id>6</id><_current_value>5</_current_value></hotrect>
          <hotrect name=\"TM_text_container\"><width>600</width><height>400</height>
            <text name=\"TM_text\"><id>0</id><wrapwidth>595</wrapwidth></text></hotrect>
          <hotrect name=\"TM_next_button\"><id>1</id><target>&true;</target><string>&-sNext;</string></hotrect>
          <hotrect name=\"TM_prev_button\"><id>2</id><target>&true;</target><string>&-sPrevious;</string></hotrect>
          <hotrect name=\"TM_close_button\"><id>3</id><target>&true;</target><width>100</width><height>30</height>
            <string>x</string></hotrect>
          <text name=\"TM_VDSG_text\"><id>7</id><string>x</string></text>
        </rect></menu>";

    fn opened(text: &str) -> (Ui, TutorialMenu) {
        let mut ui = test_support::ui();
        let mut m = TutorialMenu::new(0, 19);
        m.menu = test_support::load(&mut ui, XML, &mut m);
        m.open(
            &mut ui,
            &Message {
                title: "Hacking".into(),
                text: text.into(),
            },
        );
        (ui, m)
    }

    #[test]
    fn opening_fills_it_with_the_message() {
        let (mut ui, m) = opened("Some terminals are protected.");
        let tile = |i: usize| m.tiles[i].unwrap();
        assert_eq!(ui.string(tile(tile::TITLE), t::STRING).unwrap(), "Hacking");
        assert_eq!(
            ui.string(tile(tile::TEXT), t::STRING).unwrap(),
            "Some terminals are protected."
        );
        assert_eq!(ui.number(tile(tile::TEXT), t::ISHTML), 0.0);
        assert_eq!(ui.string(tile(tile::CLOSE), t::STRING).unwrap(), "Close");
        assert_eq!(
            ui.string(tile(tile::VDSG_TEXT), t::STRING).unwrap(),
            "VDSG MANUAL"
        );
        // One message: no pages, Next and Previous hidden and not
        // clickable.
        for i in [tile::NEXT, tile::PREVIOUS, tile::PAGE] {
            assert_eq!(ui.number(tile(i), t::VISIBLE), 0.0, "tile {i}");
        }
        assert_eq!(ui.number(tile(tile::NEXT), t::TARGET), 0.0);
        assert_eq!(ui.number(tile(tile::PREVIOUS), t::TARGET), 0.0);
        let current = ui.names.lookup_or_add("_current_value").unwrap();
        assert_eq!(ui.number(tile(tile::SCROLLBAR), current), 0.0);
        assert_eq!(ui.number(m.menu, t::VISIBLE), 1.0);
        assert_eq!(ui.number(m.menu, menu::LEAVE_STACK), 0.0);
    }

    #[test]
    fn a_text_starting_with_a_tag_is_html() {
        let (mut ui, m) = opened("<p>Hi</p>");
        assert_eq!(ui.number(m.tiles[tile::TEXT].unwrap(), t::ISHTML), 1.0);
    }

    #[test]
    fn close_and_the_cancel_code_close_it() {
        let (mut ui, mut m) = opened("Text");
        m.click(&mut ui, tile::NEXT as i32, None, 0.0);
        m.click(&mut ui, tile::PREVIOUS as i32, None, 0.0);
        assert!(!m.closed);
        assert!(!m.special_key(&mut ui, crate::menu::special::A, 0.0));
        m.click(&mut ui, tile::CLOSE as i32, None, 0.0);
        assert!(m.closed);
        assert_eq!(ui.number(m.menu, menu::LEAVE_STACK), 1.0);
        let (mut ui, mut m) = opened("Text");
        assert!(m.special_key(&mut ui, CLOSE_CODE, 0.0));
        assert!(m.closed);
    }
}
