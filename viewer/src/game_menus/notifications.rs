//! User-requested deviation from retail: acknowledgement-only boxes become
//! right-side cards, visible for five real seconds without entering menu mode.
//! Text, fonts, pictures and frame still come from message_menu.xml (the
//! original filling path is documented in ui::menus::message).

use std::collections::VecDeque;

use cellview::Game;
use ui::draw::{DrawItem, DrawKind};
use ui::menus::message::{MessageBox, MessageMenu, FILE};
use ui::names::t;

use super::{Files, Screen};

const SECONDS: f32 = 5.0;
const VISIBLE: usize = 3;
const SCALE: f32 = 0.75;
const GAP: f32 = 12.0;
const TOP: f32 = 70.0;

#[derive(Default)]
pub(super) struct Notifications {
    template: Option<MessageMenu>,
    cards: VecDeque<Card>,
}

struct Card {
    items: Vec<DrawItem>,
    bounds: [f32; 4],
    remaining: f32,
    shown: bool,
}

pub(super) fn is_notice(b: &MessageBox, localized_ok: Option<&str>) -> bool {
    let mut labels = b.buttons.iter().filter(|s| !s.is_empty());
    let Some(label) = labels.next() else {
        return true;
    };
    labels.next().is_none()
        && (label.eq_ignore_ascii_case("OK") || localized_ok == Some(label.as_str()))
}

/// Build through the existing game XML, using one reusable tile tree. Cards
/// cache drawing only: they never join the input/menu stack or capture focus.
pub(super) fn show(screen: &mut Screen, game: &Game, mut b: MessageBox) -> bool {
    let mut menu = match screen.notifications.template.take() {
        Some(m) => m,
        None => {
            let mut m = MessageMenu::new(0);
            match screen.load(game, FILE, &mut m) {
                Ok(tile) => m.menu = tile,
                Err(e) => {
                    println!("Notification card can't be shown: {e}");
                    return false;
                }
            }
            m
        }
    };
    b.buttons.clear();
    b.background_alpha = b.background_alpha.min(160.0);
    // Width is in physical pixels; the XML's wrapping is in menu units.
    b.width = (screen.size.x as f32 * 0.3 / SCALE) as u32;
    menu.queue.clear();
    menu.push(b);
    menu.open(&mut screen.ui, None);
    if let Some(list) = menu.tiles[4] {
        screen.ui.set_number(list, t::VISIBLE, 0.0);
        screen.ui.set_number(list, t::HEIGHT, 0.0);
    }
    screen.ui.refresh();
    let mut files = Files {
        game,
        sizes: &mut screen.sizes,
        atlases: &mut screen.atlases,
    };
    ui::draw::update_file_sizes(&mut screen.ui, menu.menu, &mut files);
    let items = ui::draw_list(&mut screen.ui, menu.menu, &mut files, &|_| None);
    // Informational cards are silent; don't retain the modal opening sounds
    // on the reusable template.
    menu.sounds.clear();
    screen.ui.set_number(menu.menu, t::VISIBLE, 0.0);
    screen.notifications.template = Some(menu);
    let Some(bounds) = bounds(&items) else {
        return false;
    };
    screen.notifications.cards.push_back(Card {
        items,
        bounds,
        remaining: SECONDS,
        shown: false,
    });
    true
}

fn rects(item: &DrawItem) -> Vec<[f32; 4]> {
    match &item.kind {
        DrawKind::Image { rect, .. } => vec![*rect],
        DrawKind::Text { glyphs, .. } => glyphs.iter().map(|g| g.0).collect(),
        DrawKind::Model { triangles, .. } => triangles
            .iter()
            .flatten()
            .map(|(p, _)| [p[0], p[1], 0.0, 0.0])
            .collect(),
    }
}

fn bounds(items: &[DrawItem]) -> Option<[f32; 4]> {
    let mut all = items.iter().flat_map(rects);
    let first = all.next()?;
    let mut min = [first[0], first[1]];
    let mut max = [first[0] + first[2], first[1] + first[3]];
    for r in all {
        for i in 0..2 {
            min[i] = min[i].min(r[i]);
            max[i] = max[i].max(r[i] + r[i + 2]);
        }
    }
    Some([min[0], min[1], max[0] - min[0], max[1] - min[1]])
}

impl Notifications {
    pub(super) fn move_cards(&mut self, from: &mut Self) {
        self.cards = std::mem::take(&mut from.cards);
    }

    pub(super) fn draw(&mut self, dt: f32, ui: &ui::Ui) -> Vec<DrawItem> {
        // Queued cards get their full five seconds after becoming visible.
        for c in &mut self.cards {
            if c.shown {
                c.remaining -= dt;
            }
            c.shown = false;
        }
        self.cards.retain(|c| c.remaining > 0.0);
        let mut out = Vec::new();
        let margin = ui.screen_size.safe_x.max(20.0);
        // Leave the top status line and FPS display unobstructed.
        let top = ui.screen_size.safe_y.max(TOP);
        let mut y = top;
        for c in self.cards.iter_mut().take(VISIBLE) {
            // Keep long notices readable and queue overflow rather than
            // squeezing subsequent cards into the space left below them.
            let scale = SCALE
                .min((ui.screen_size.height() - top - margin) / c.bounds[3].max(1.0))
                .min((ui.screen_size.width() - 2.0 * margin) / c.bounds[2].max(1.0));
            if y + c.bounds[3] * scale > ui.screen_size.height() - margin {
                break;
            }
            let x = ui.screen_size.width() - margin - c.bounds[2] * scale;
            let offset = [x - c.bounds[0] * scale, y - c.bounds[1] * scale];
            let move_rect = |r: &mut [f32; 4]| {
                for i in 0..2 {
                    r[i] = r[i] * scale + offset[i];
                    r[i + 2] *= scale;
                }
            };
            for mut item in c.items.clone() {
                // Keep cards below actual interactive menus.
                item.depth = 1.0;
                match &mut item.kind {
                    DrawKind::Image { rect, .. } => move_rect(rect),
                    DrawKind::Text { glyphs, .. } => {
                        for g in glyphs {
                            move_rect(&mut g.0);
                        }
                    }
                    DrawKind::Model { triangles, .. } => {
                        for (p, _) in triangles.iter_mut().flatten() {
                            for i in 0..2 {
                                p[i] = p[i] * scale + offset[i];
                            }
                        }
                    }
                }
                out.push(item);
            }
            c.shown = true;
            y += c.bounds[3] * scale + GAP;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledgements_only_preserve_choices_and_localization() {
        let make = |buttons: &[&str]| MessageBox::new("text", None, buttons, 0x17, 200.0);
        assert!(is_notice(&make(&["OK"]), None));
        assert!(is_notice(&make(&["&OK", " "]), None));
        assert!(is_notice(&make(&["D'accord"]), Some("D'accord")));
        assert!(!is_notice(&make(&["Yes", "No"]), None));
        assert!(!is_notice(&make(&["Continue"]), None));
    }

    #[test]
    fn overflow_waits_and_visible_cards_expire_after_five_seconds() {
        let ui = ui::Ui::new(
            ui::Screen {
                width_px: 1920,
                height_px: 1080,
                safe_x: 15.0,
                safe_y: 15.0,
            },
            ui::SystemColors::new(None, None),
            Box::new(|_| None),
        );
        let mut notices = Notifications::default();
        for _ in 0..4 {
            let item = DrawItem {
                tile: 0,
                depth: 0.0,
                color: [1.0; 4],
                kind: DrawKind::Image {
                    texture: "test".into(),
                    rect: [0.0, 0.0, 400.0, 100.0],
                    uv: [0.0, 0.0, 1.0, 1.0],
                    repeat_u: false,
                    scroll: None,
                },
            };
            notices.cards.push_back(Card {
                bounds: bounds(std::slice::from_ref(&item)).unwrap(),
                items: vec![item],
                remaining: SECONDS,
                shown: false,
            });
        }
        let draws = notices.draw(1.0, &ui);
        assert_eq!(draws.len(), 3);
        let DrawKind::Image { rect, .. } = &draws[0].kind else {
            panic!()
        };
        assert_eq!(rect[0] + rect[2], ui.screen_size.width() - 20.0);
        assert_eq!(notices.draw(4.99, &ui).len(), 3);
        assert_eq!(notices.draw(0.02, &ui).len(), 1);
        assert_eq!(notices.cards[0].remaining, SECONDS);
        assert!(notices.draw(5.0, &ui).is_empty());
    }
}
