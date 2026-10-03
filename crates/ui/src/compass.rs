//! The compass strip (`00779070`) for menus other than the HUD that carry
//! one: V.A.T.S.'s hit points bracket builds the same compass window from
//! `HUDTemplates.xml` and the game updates it with the same function
//! (`007ec810` calls `00779070` with the V.A.T.S. menu's tiles). The rules
//! are the HUD's (`crate::hud`, where they're described); this takes the
//! tiles as arguments.

use crate::hud::{bearing, CompassActor, CompassMarker};
use crate::names::t;
use crate::tile::{TileId, Ui};

/// A compass window's pieces (the setup's `+0x78`…`+0x84` in V.A.T.S.,
/// `+0x34`…`+0x3c` and the window in the HUD).
#[derive(Debug, Clone, Default)]
pub struct CompassTiles {
    pub window: TileId,
    pub markers: Vec<TileId>,
    pub quests: Vec<TileId>,
    pub player: Option<TileId>,
    pub npcs: Vec<TileId>,
    /// The `_Heading` and `_Distance` user traits.
    pub heading_trait: i32,
    pub distance_trait: i32,
}

/// What the compass shows.
#[derive(Debug, Clone, Default)]
pub struct CompassInput {
    /// Degrees clockwise from north.
    pub heading: f32,
    pub position: [f32; 3],
    pub interior: bool,
    pub markers: Vec<CompassMarker>,
    pub actors: Vec<CompassActor>,
    /// Alpha for full strength (`fHudOpacity` × 255).
    pub opacity: f32,
}

fn distance_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// One frame of the compass (`00779070`); returns the strip's scroll
/// (`TexScroll`: x offset, y, scale x, y).
pub fn update(ui: &mut Ui, tiles: &CompassTiles, input: &CompassInput) -> [f32; 4] {
    let opacity = input.opacity;
    let heading = input.heading.rem_euclid(360.0);
    let mut offset = heading / 360.0 - 0.15;
    if offset < 0.0 {
        offset += 1.0;
    }
    let compass_w = ui.number(tiles.window, t::WIDTH);
    let half = (compass_w / 2.0) as i32 - 35;
    let limit = 70.0f32;
    let fade_from = half as f32 * 0.8 * 2.0;
    let fade_to = (half * 2) as f32;
    let place = |rel: f32| -> f32 { (2 * half) as f32 * ((rel + limit) / (2.0 * limit)) };
    let fade = |x: f32| -> f32 {
        if x <= fade_from {
            opacity
        } else {
            opacity * (1.0 - (x - fade_from) / (fade_to - fade_from))
        }
    };
    let reach = 20000.0f32;
    let mut markers = input
        .markers
        .iter()
        .filter(|m| distance_sq(m.position, input.position) <= reach * reach);
    for &icon in &tiles.markers {
        let Some(m) = markers.next() else {
            ui.set_number(icon, t::VISIBLE, 0.0);
            continue;
        };
        let file = if m.found {
            "Interface\\HUD\\glow_hud_compass_landmark_discovered.dds"
        } else {
            "Interface\\HUD\\glow_hud_compass_landmark.dds"
        };
        ui.set_string(icon, t::FILENAME, file);
        let rel = bearing(m.position, input.position, heading);
        if rel.abs() >= limit {
            ui.set_number(icon, t::VISIBLE, 0.0);
            continue;
        }
        let x = place(rel);
        ui.set_number(icon, tiles.heading_trait, x);
        ui.set_number(
            icon,
            tiles.distance_trait,
            distance_sq(m.position, input.position).sqrt(),
        );
        ui.set_number(icon, t::VISIBLE, 1.0);
        ui.set_number(icon, t::ALPHA, fade(x));
    }
    let reach = if input.interior { 1500.0 } else { 3000.0 };
    let mut actors = input
        .actors
        .iter()
        .filter(|a| distance_sq(a.position, input.position) <= reach * reach);
    for &icon in &tiles.npcs {
        let Some(a) = actors.next() else {
            ui.set_number(icon, t::VISIBLE, 0.0);
            continue;
        };
        let rel = bearing(a.position, input.position, heading);
        if rel.abs() >= limit {
            ui.set_number(icon, t::VISIBLE, 0.0);
            continue;
        }
        let x = place(rel);
        ui.set_number(icon, tiles.heading_trait, x + 2.0);
        ui.set_number(icon, t::VISIBLE, 1.0);
        ui.set_number(icon, t::SYSTEMCOLOR, if a.hostile { 2.0 } else { 1.0 });
        ui.set_number(icon, t::ALPHA, fade(x));
    }
    for &icon in &tiles.quests {
        ui.set_number(icon, t::VISIBLE, 0.0);
    }
    if let Some(p) = tiles.player {
        ui.set_number(p, t::VISIBLE, 0.0);
    }
    [offset, 0.0, 1.0, 1.0]
}
