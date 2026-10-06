//! The game's HUD on screen (`ui::hud`): health and action points with
//! their meters, the compass with map markers and nearby people, the
//! crosshair, the weapon's ammunition and condition, and the message corner
//! (top left), laid out and filled as the game's code does from its own
//! menu file (`menus\main\hud_main_menu.xml`), fonts and textures.
//!
//! Drawn as the game draws it (read from a recording of a frame in Doc
//! Mitchell's house): every piece a quad, its texture × its colour
//! (`TILE1000.pso`; the compass `TILE1001`, the strip scrolled by the
//! heading and its alpha × an alpha map), blended source alpha over inverse
//! source alpha onto the finished picture's stored values after the image
//! space pass, texels read bilinearly from the top level only. Here the
//! pieces go into a picture of their own first (a 2D camera, transparent
//! black underneath, stored values, blended the same way), which the image
//! space pass's last step lays over the scene before turning stored values
//! into linear light (`grade`): the same as drawing each piece onto the
//! scene. Direct3D 9 puts pixel centres on whole coordinates, so the game's
//! quads land half a pixel right of and below where the same numbers land
//! here; the quads are moved by that half pixel.
//!
//! Not yet: the HUD's other parts (sneak meter, enemy health, quest
//! reminders, the XP meter, radiation, hardcore needs;
//! `ui::hud::NOT_FOLLOWED`), subtitles, and the message icons the game
//! picks per kind of message (all get the neutral Vault Boy).

// The shader-layout derive generates checking functions the compiler
// reports as unused.
#![allow(dead_code)]

use std::collections::HashMap;

use bevy::asset::{load_internal_asset, weak_handle, RenderAssetUsages};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat, TextureUsages,
    WgpuFeatures,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::RenderLayers;
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::window::PrimaryWindow;
use cellview::{Game, TextureData};
use ui::draw::{DrawItem, DrawKind, Textures};
use ui::hud::{CompassActor, CompassMarker, WeaponState};
use ui::names::t;
use world::dialogue::PLAYER_REF;

use crate::dialogue::{Conversation, DialogueState, TalkTarget, Talkers};
use crate::{FlyCamera, GameFiles};

const SHADER: Handle<Shader> = weak_handle!("3c8e51d2-7a4f-4b19-9e06-5d2b8f17a6c3");

/// The render layer only the HUD's camera sees.
const HUD_LAYER: usize = 23;

/// Ordinary action classes (`00579280`, `00f80050`). Ownership is applied
/// separately, using the same crime rules as activation (`00579690`).
fn supported_activation(
    order: &esm::LoadOrder,
    reference: esm::FormId,
) -> Option<(esm::FormId, &'static str, &'static str)> {
    let base = world::scripting::base_of(order, reference)?;
    let rr = order.get(base)?;
    let (setting, default) = match rr.entry.header.kind {
        kind if kind == esm::FourCC::new(b"ACTI") => ("sTargetTypeActivate", "Activate"),
        kind if kind == esm::FourCC::new(b"CONT") => ("sTargetTypeOpen", "Open"),
        kind if kind == esm::FourCC::new(b"BOOK") => ("sTargetTypeRead", "Read"),
        kind if world::scripting::is_item(kind) => ("sTargetTypeTake", "Take"),
        // Furniture Sit/Sleep and terminal state branches remain unported.
        _ => return None,
    };
    Some((base, setting, default))
}

/// The HUD's picture, which the image space pass lays over the scene.
#[derive(Resource, Clone, ExtractResource)]
pub struct HudLayer(pub Handle<Image>);

/// The HUD's camera (it writes the HUD's picture each frame; while a menu's
/// 3D scene is drawn into it first it blends over it: `caravan_table`).
#[derive(Component)]
pub struct HudCamera;

/// Whether the game's HUD is drawn (`--no-hud` turns it off).
#[derive(Resource)]
pub struct ShowHud(pub bool);

/// Messages for the HUD's corner, from what scripts and the game announce
/// (`scripts`); `on` while the HUD shows them (otherwise they go to the
/// old notice panel).
#[derive(Resource, Default)]
pub struct HudMessages {
    pub on: bool,
    pub queue: Vec<String>,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, SHADER, "hud.wgsl", Shader::from_wgsl);
        app.add_plugins((
            Material2dPlugin::<TileMaterial>::default(),
            ExtractResourcePlugin::<HudLayer>::default(),
        ))
        .init_resource::<HudMessages>()
        .init_resource::<GameHud>()
        .add_systems(Startup, setup_hud_layer)
        .add_systems(Update, update_hud.after(crate::scripts::run_scripts));
    }
}

/// What a tile shader takes (`TintColor`, `TexScroll`), and which one.
#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TileParams {
    pub(crate) tint: Vec4,
    pub(crate) scroll: Vec4,
    /// `x`: 1 for `TILE1001` (scroll and alpha map), 0 for `TILE1000`.
    pub(crate) mode: Vec4,
}

/// One HUD piece's material: its texture, colour and shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TileMaterial {
    #[uniform(0)]
    pub(crate) params: TileParams,
    #[texture(1)]
    #[sampler(2)]
    pub(crate) texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub(crate) alpha_map: Handle<Image>,
}

impl TileMaterial {
    /// A piece drawn with `TILE1000` (its texture × its colour), for the
    /// other menus' pictures (the Pip-Boy's); `white` the alpha map left
    /// unused.
    pub(crate) fn plain(tint: Vec4, texture: Handle<Image>, white: Handle<Image>) -> Self {
        TileMaterial {
            params: TileParams {
                tint,
                scroll: Vec4::new(0.0, 0.0, 1.0, 1.0),
                mode: Vec4::ZERO,
            },
            texture,
            alpha_map: white,
        }
    }
}

impl Material2d for TileMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        // Source alpha over inverse source alpha, as the game's HUD.
        AlphaMode2d::Blend
    }
}

/// The HUD as built for the window's size, and what's on screen of it.
struct Built {
    ui: ui::Ui,
    hud: ui::Hud,
    size: UVec2,
    opacity: f32,
    sizes: HashMap<String, Option<(u32, u32)>>,
    atlases: HashMap<String, Option<ui::Atlas>>,
    /// Pictures by path and how they're addressed (repeating across,
    /// repeating down).
    images: HashMap<(String, bool, bool), Option<Handle<Image>>>,
    /// Font pictures by font (1 to 8) and picture number.
    font_images: HashMap<(usize, u32), Option<Handle<Image>>>,
    /// The draw list on screen, and its entities and assets.
    last: Vec<DrawItem>,
    drawn: Vec<(Entity, Handle<Mesh>, Handle<TileMaterial>)>,
    /// V.A.T.S.'s menu (`ui::vats`), laid out with the HUD, and what the
    /// HUD's mask hid while V.A.T.S. is on.
    vats: Option<ui::vats::VatsMenu>,
    masked: ui::hud::Masked,
}

/// The game's HUD, once built (`failed` when its files can't be read).
#[derive(Resource, Default)]
pub struct GameHud {
    built: Option<Box<Built>>,
    failed: bool,
    /// A white pixel: the alpha map of pieces without one.
    white: Option<Handle<Image>>,
}

/// The HUD's picture and its camera, at the window's size.
fn setup_hud_layer(
    mut commands: Commands,
    show: Res<ShowHud>,
    mut images: ResMut<Assets<Image>>,
    mut hud: ResMut<GameHud>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !show.0 {
        return;
    }
    let size = windows
        .single()
        .map(|w| UVec2::new(w.physical_width(), w.physical_height()))
        .unwrap_or(UVec2::new(1920, 1080))
        .max(UVec2::ONE);
    let mut image = Image::new_uninit(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba16Float,
        // Kept in the main world too: the camera reads its size there.
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let layer = images.add(image);
    commands.insert_resource(HudLayer(layer.clone()));
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::from(layer),
            // Before the main camera, whose last pass reads the picture,
            // and before the lockpicking menu's cameras, which draw onto it
            // (`lockpick`).
            order: -4,
            // A float picture: stored values kept as they are, blended as
            // they are.
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Off,
        RenderLayers::layer(HUD_LAYER),
        HudCamera,
    ));
    let mut white = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    white.sampler = sampler(false, false);
    hud.white = Some(images.add(white));
}

/// Bilinear, the top level only (the game's `MIPFILTER` none), repeating
/// or clamped.
pub(crate) fn sampler(repeat_u: bool, repeat_v: bool) -> ImageSampler {
    let mode = |repeat| {
        if repeat {
            ImageAddressMode::Repeat
        } else {
            ImageAddressMode::ClampToEdge
        }
    };
    ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: mode(repeat_u),
        address_mode_v: mode(repeat_v),
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    })
}

/// Every text game setting (`GMST` named `s…`), for `&-sName;`.
pub(crate) fn text_settings(order: &esm::LoadOrder) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for rr in order.records_of_type(esm::FourCC::new(b"GMST")) {
        let Ok(record) = rr.record() else {
            continue;
        };
        let Some(name) = record.get(esm::sig::EDID).map(|s| s.zstring()) else {
            continue;
        };
        if name.starts_with('s') {
            if let Some(text) = record.get(esm::sig::DATA).map(|s| s.zstring()) {
                out.insert(name, text);
            }
        }
    }
    out
}

/// Texture sizes and atlases from the game's files, for laying pieces out.
pub(crate) struct Files<'a> {
    pub(crate) game: &'a Game,
    pub(crate) sizes: &'a mut HashMap<String, Option<(u32, u32)>>,
    pub(crate) atlases: &'a mut HashMap<String, Option<ui::Atlas>>,
}

impl Textures for Files<'_> {
    fn size(&mut self, path: &str) -> Option<(u32, u32)> {
        let game = self.game;
        *self.sizes.entry(path.to_string()).or_insert_with(|| {
            let bytes = game.assets.read(path).ok().flatten()?;
            let t = TextureData::from_dds(path, bytes).ok()?;
            Some((t.width, t.height))
        })
    }

    fn atlas(&mut self, path: &str) -> Option<ui::Atlas> {
        let game = self.game;
        self.atlases
            .entry(path.to_string())
            .or_insert_with(|| {
                let bytes = game.assets.read(path).ok().flatten()?;
                Some(ui::Atlas::parse(&String::from_utf8_lossy(&bytes)))
            })
            .clone()
    }
}

/// A picture as the game samples it: stored values (no sRGB decoding),
/// the top level only, block-compressed when the card takes it.
pub(crate) fn upload_picture(
    images: &mut Assets<Image>,
    game: &Game,
    path: &str,
    repeat: (bool, bool),
    compressed: bool,
) -> Option<Handle<Image>> {
    let bytes = game.assets.read(path).ok().flatten()?;
    let mut texture = TextureData::from_dds(path, bytes).ok()?;
    texture.linear = true;
    texture.mip_levels = 1;
    let format = texture.gpu_format(compressed);
    let data = texture.level_data(format).ok()?;
    // (Block-compressed data has no size per pixel, so not `Image::new`.)
    let mut image = Image::new_uninit(
        Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        crate::gpu_format(format),
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(data);
    image.sampler = sampler(repeat.0, repeat.1);
    Some(images.add(image))
}

/// A font's picture (`textures\fonts\<name>.tex`: width, height, RGBA).
pub(crate) fn upload_font_picture(
    images: &mut Assets<Image>,
    game: &Game,
    path: &str,
) -> Option<Handle<Image>> {
    let bytes = game.assets.read(path).ok().flatten()?;
    let (w, h, pixels) = ui::font::read_tex(&bytes)?;
    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels.to_vec(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = sampler(false, false);
    Some(images.add(image))
}

/// Quads in menu units (x, y, width, height; texture coordinates of the
/// top-left, top-right, bottom-left and bottom-right corners) as a mesh in
/// the HUD camera's pixels: `k` pixels a unit, half a pixel right and down
/// (Direct3D 9's pixel centres), y up from the middle.
pub(crate) fn quads_mesh(quads: &[Quad], k: f32, size: UVec2) -> Mesh {
    let (w, h) = (size.x as f32, size.y as f32);
    let point = |x: f32, y: f32| [x * k + 0.5 - w / 2.0, h / 2.0 - (y * k + 0.5), 0.0];
    let mut positions = Vec::with_capacity(quads.len() * 4);
    let mut uvs = Vec::with_capacity(quads.len() * 4);
    let mut indices = Vec::with_capacity(quads.len() * 6);
    for (rect, uv) in quads {
        let [x, y, qw, qh] = *rect;
        let base = positions.len() as u32;
        positions.push(point(x, y));
        positions.push(point(x + qw, y));
        positions.push(point(x, y + qh));
        positions.push(point(x + qw, y + qh));
        uvs.extend_from_slice(uv);
        indices.extend_from_slice(&[base, base + 2, base + 1, base + 1, base + 2, base + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// What the HUD shows, read from the game's state.
#[derive(bevy::ecs::system::SystemParam)]
pub struct HudState<'w, 's> {
    time: Res<'w, Time>,
    game: Res<'w, GameFiles>,
    state: Res<'w, DialogueState>,
    attack: Res<'w, crate::combat::PlayerAttack>,
    conversation: Res<'w, Conversation>,
    talk_target: Res<'w, TalkTarget>,
    activatable: Res<'w, crate::scripts::Activatable>,
    doors: Res<'w, crate::walk::Doors>,
    collision: Res<'w, crate::walk::CellCollision>,
    menus: Res<'w, crate::menus::Menus>,
    talkers: Res<'w, Talkers>,
    markers: Res<'w, crate::map::MapMarkers>,
    exterior: Option<Res<'w, crate::exterior::Exterior>>,
    cameras: Query<'w, 's, &'static Transform, With<FlyCamera>>,
    /// The game's menus open, drawn over the HUD (`game_menus`).
    game_menus: Res<'w, crate::game_menus::MenuDraw>,
}

impl HudState<'_, '_> {
    /// The native rollover prompt, matching the viewer's E dispatch order:
    /// person, load/swing door, then a supported activatable. The Activatable text
    /// resource is deliberately ignored; its name is built from the base record.
    fn info_prompt(&self) -> Option<ui::hud::InfoPrompt> {
        if self.conversation.0.is_some()
            || self.menus.is_open()
            || !self.game_menus.1.is_empty()
            || self.state.0.controls_off[world::scripting::controls::ROLLOVER]
            || self.state.0.controls_off[world::scripting::controls::MOVEMENT]
        {
            return None;
        }
        let order = &self.game.0.order;
        let camera = self.cameras.single().ok()?;
        let eye = crate::walk::game_point(camera.translation);
        let f = camera.forward().as_vec3();
        let direction = [f.x, -f.z, f.y];
        let choice = if let Some((talker, _)) = &self.talk_target.0 {
            Some((talker.base, "sTargetTypeTalk", "Talk", false))
        } else if let Some(door) =
            crate::walk::door_in_view(&self.doors.0, &self.collision.0, eye, direction)
        {
            Some((
                esm::FormId(door.reference),
                "sTargetTypeOpenDoor",
                "Open",
                true,
            ))
        } else if let Some(reference) =
            crate::walk::opening_door_in_view(&self.collision.0, eye, direction)
        {
            Some((reference, "sTargetTypeOpen", "Open", true))
        } else {
            self.activatable.0.as_ref().and_then(|(reference, _)| {
                let (base, setting, default) = supported_activation(order, *reference)?;
                Some((base, setting, default, false))
            })
        }?;
        let (base, setting, exe_default, resolve_target_base) = choice;
        let base = if resolve_target_base {
            world::scripting::base_of(order, base)?
        } else {
            base
        };
        let target = order.get(base)?.record().ok()?.full_name()?;
        let crime = self.activatable.0.as_ref().is_some_and(|(reference, _)| {
            world::scripting::base_of(order, *reference) == Some(base)
                && order.get(base).is_some_and(|r| {
                    world::scripting::is_item(r.entry.header.kind)
                        || r.entry.header.kind == esm::FourCC::new(b"CONT")
                })
                && !world::crime::may_take(
                    order,
                    &self.state.0,
                    world::crime::owner_of(order, &self.state.0, *reference),
                )
        });
        let (setting, exe_default) = if crime {
            ("sSteal", "Steal")
        } else {
            (setting, exe_default)
        };
        let action = world::scripting::game_setting_text(order, setting)
            .unwrap_or_else(|| exe_default.to_string());
        Some(ui::hud::InfoPrompt {
            action,
            target,
            // The current viewer action handler is KeyE; display that same
            // input until key rebinding is implemented end to end.
            shortcut: Some("E".to_string()),
            crime,
        })
    }

    /// The HUD's input this frame (`opacity`: `fHudOpacity`).
    fn input(&self, opacity: f32) -> ui::HudInput {
        use world::combat;
        let order = &self.game.0.order;
        let state = &self.state.0;
        let facts = world::scripting::Facts {
            order,
            state,
            speaker: None,
        };
        // Action points: actor value 12.
        let action_points = facts.current_actor_value(PLAYER_REF, 12).unwrap_or(0.0) as f32;
        let action_points_max = facts.permanent_actor_value(PLAYER_REF, 12).unwrap_or(0.0) as f32;
        let weapon = combat::weapon_in_hand(order, state, PLAYER_REF).map(|w| {
            // The counter: rounds in the clip / the rest carried (`%i/%i`,
            // `007721c0`).
            let ammo = w.ammo_in_use(order, state, PLAYER_REF).map(|a| {
                let held = state.item_count(order, PLAYER_REF, a).max(0);
                let clip = (self.attack.in_clip().unwrap_or(w.clip) as i32).min(held);
                (clip, held - clip)
            });
            WeaponState {
                id: w.form_id.0,
                ammo,
                condition: combat::weapon_condition(state, PLAYER_REF, w.form_id),
            }
        });
        let (eye, heading) = self
            .cameras
            .single()
            .map(|t| {
                let f = t.forward().as_vec3();
                (
                    crate::walk::game_point(t.translation),
                    f.x.atan2(-f.z).to_degrees(),
                )
            })
            .unwrap_or_default();
        let position =
            state
                .player_position
                .unwrap_or([eye[0], eye[1], eye[2] - cellview::EYE_HEIGHT]);
        let outdoors = self.exterior.is_some();
        let markers = if outdoors {
            self.markers
                .list
                .iter()
                .map(|m| CompassMarker {
                    position: m.position,
                    found: state.discovered.contains(&m.reference),
                })
                .collect()
        } else {
            Vec::new()
        };
        // People: red when fighting the player or set to attack them on
        // sight (a guess at the game's "hostile").
        let actors = self
            .talkers
            .0
            .iter()
            .filter(|t| !state.dead.contains(&t.reference))
            .map(|t| CompassActor {
                position: t.position,
                hostile: state.combat.get(&t.reference) == Some(&PLAYER_REF)
                    || world::factions::attacks_on_sight(order, state, t.reference, PLAYER_REF),
            })
            .collect();
        let dead = state.dead.contains(&PLAYER_REF);
        // The XP meter's levels, from the game's settings as world reads
        // them; "in combat" as IsInCombat answers for the player.
        let base = world::experience::xp_for_level(order, 2) as i32;
        let bump = world::experience::xp_for_level(order, 3) as i32 - 2 * base;
        let experience = ui::Experience {
            xp: world::experience::xp(state) as i32,
            level: i32::from(state.player_level),
            max_level: i32::from(world::experience::max_level(order)),
            base,
            bump,
            level_up_ready: world::experience::level_up_ready(state),
            in_combat: state.combat.contains_key(&PLAYER_REF)
                || state.combat.values().any(|t| *t == PLAYER_REF),
        };
        ui::HudInput {
            time: self.time.elapsed_secs(),
            health: combat::health(order, state, PLAYER_REF).unwrap_or(0.0) as f32,
            health_max: combat::max_health(order, state, PLAYER_REF).unwrap_or(0.0) as f32,
            dead,
            action_points,
            action_points_max,
            weapon,
            heading,
            position,
            interior: !outdoors,
            markers,
            actors,
            opacity,
            crosshair: !dead,
            subtitle: None,
            experience: Some(experience),
            menu_open: self.menu_hides_xp(),
        }
    }

    /// Whether the HUD shows: not in a menu or the dialogue menu (a line
    /// said in passing keeps it). With the game's own menus open, the HUD
    /// shows the pieces those menus leave on (`parts`).
    fn shown(&self) -> bool {
        if let Some(&first) = self.game_menus.1.first() {
            return ui::hud::parts_for_menu(Some(first)) != 0;
        }
        !self.menus.is_open()
            && self
                .conversation
                .0
                .as_ref()
                .is_none_or(|t| t.is_line_only())
    }

    /// The HUD's pieces left on (`ui::hud::parts_for_menu`, by the game's
    /// menu that opened from the game), and whether it's the dialogue menu.
    fn parts(&self) -> (u32, bool) {
        let first = self.game_menus.1.first().copied();
        let parts = if first.is_none() {
            ui::hud::gameplay_parts(
                self.state.0.controls_off[world::scripting::controls::MOVEMENT],
                self.state.0.controls_off[world::scripting::controls::ROLLOVER],
            )
        } else {
            ui::hud::parts_for_menu(first)
        };
        (parts, first == Some(ui::menus::dialog::CLASS))
    }

    /// A menu is open that the XP meter hides for (`0077c4e0`: any but a
    /// container, hacking, a terminal or dialogue).
    fn menu_hides_xp(&self) -> bool {
        let game = self
            .game_menus
            .1
            .iter()
            .any(|c| !matches!(c, 1008 | 1055 | 1057 | 1009));
        let old = self.game_menus.1.is_empty() && self.menus.is_open();
        game || old
    }
}

/// Builds the HUD for a screen size from the game's files.
fn build(game: &Game, size: UVec2) -> Result<Built, String> {
    let mut read = |p: &str| game.assets.read(p).ok().flatten();
    let ini = |s: &str, k: &str| game.settings.get(s, k).map(str::to_string);
    // The exe's own words for the settings V.A.T.S.'s menu shows that the
    // data doesn't set.
    let mut texts = text_settings(&game.order);
    for (name, text) in ui::vats::EXE_TEXT {
        texts
            .entry(name.to_string())
            .or_insert_with(|| text.to_string());
    }
    let mut ui = ui::game::new_ui(&mut read, &ini, texts, size.x, size.y);
    let hud = ui::hud::load(&mut ui, &mut read)?;
    let vats = match ui::vats::load(&mut ui, &mut read) {
        Ok(v) => Some(v),
        Err(e) => {
            println!("  V.A.T.S.'s menu can't be shown: {e}");
            None
        }
    };
    for w in &ui.warnings {
        println!("  HUD: {w}");
    }
    let opacity = game
        .settings
        .float("Interface", "fHudOpacity")
        .unwrap_or(1.0);
    Ok(Built {
        ui,
        hud,
        size,
        opacity,
        sizes: HashMap::new(),
        atlases: HashMap::new(),
        images: HashMap::new(),
        font_images: HashMap::new(),
        last: Vec::new(),
        drawn: Vec::new(),
        vats,
        masked: ui::hud::Masked::default(),
    })
}

/// A quad: x, y, width, height in menu units, and the texture coordinates
/// of its top-left, top-right, bottom-left and bottom-right corners.
pub(crate) type Quad = ([f32; 4], [[f32; 2]; 4]);

/// One draw: a texture, the alpha map (the compass), the scroll, the quads.
struct Piece {
    texture: Handle<Image>,
    alpha_map: Option<Handle<Image>>,
    scroll: [f32; 4],
    quads: Vec<Quad>,
}

/// Where the HUD's pieces go.
#[derive(bevy::ecs::system::SystemParam)]
pub struct HudAssets<'w> {
    images: ResMut<'w, Assets<Image>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<TileMaterial>>,
    device: Option<Res<'w, RenderDevice>>,
}

/// Each frame: the HUD filled from the game's state, and its pieces put on
/// screen when they've changed.
#[allow(clippy::too_many_arguments)]
fn update_hud(
    mut commands: Commands,
    show: Res<ShowHud>,
    mut hud: ResMut<GameHud>,
    mut messages: ResMut<HudMessages>,
    layer: Option<Res<HudLayer>>,
    from: HudState,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: HudAssets,
    mut old_line: Query<&mut Visibility, With<crate::combat::HudText>>,
    vats: Res<crate::vats::Vats>,
) {
    let HudAssets {
        mut images,
        mut meshes,
        mut materials,
        device,
    } = assets;
    let (Some(layer), true) = (layer, show.0) else {
        return;
    };
    if hud.failed {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let size = UVec2::new(window.physical_width(), window.physical_height());
    if size.x == 0 || size.y == 0 {
        return;
    }
    let game = &from.game.0;
    let GameHud {
        built,
        failed,
        white,
    } = &mut *hud;
    if built.as_ref().is_none_or(|b| b.size != size) {
        // A new size lays everything out again.
        if let Some(old) = built.take() {
            for (e, mesh, material) in old.drawn {
                commands.entity(e).despawn();
                meshes.remove(&mesh);
                materials.remove(&material);
            }
        }
        match build(game, size) {
            Ok(b) => *built = Some(Box::new(b)),
            Err(e) => {
                println!("The game's HUD can't be shown: {e}");
                *failed = true;
                return;
            }
        }
        let wanted = Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        if images
            .get(&layer.0)
            .is_some_and(|i| i.texture_descriptor.size != wanted)
        {
            if let Some(image) = images.get_mut(&layer.0) {
                image.texture_descriptor.size = wanted;
            }
        }
        messages.on = true;
    }
    let Some(b) = built.as_mut() else {
        return;
    };
    let Some(white) = white.clone() else {
        return;
    };

    // The old health line gives way, except to say the player is dead.
    let dead = from.state.0.dead.contains(&PLAYER_REF);
    for mut v in &mut old_line {
        let want = if dead {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
    }

    // Experience goes to the XP meter (`world` announces it as "XP +N",
    // `sStatsXP`), as does "LEVEL UP"; the rest to the message corner.
    let xp_gain = format!(
        "{} +",
        b.ui.setting_text("sStatsXP").unwrap_or_else(|| "XP".into())
    );
    let level_up = b.ui.setting_text("sLevelUp");
    for text in messages.queue.drain(..) {
        if let Some(n) = text
            .strip_prefix(&xp_gain)
            .and_then(|n| n.trim().parse::<f64>().ok())
        {
            b.hud.add_experience(n as i32);
        } else if level_up.as_deref() != Some(text.as_str()) {
            b.hud
                .queue_message(&mut b.ui, &text, None, ui::hud::MESSAGE_SECONDS);
        }
    }
    let input = from.input(b.opacity);
    // Restore each prior mask before this frame writes tile visibility, then
    // save the fresh state under the mask selected below.
    b.hud.lift_mask(&mut b.ui, &mut b.masked);
    b.hud.update(&mut b.ui, &input);
    let info = from.info_prompt();
    b.hud.update_info(&mut b.ui, info.as_ref(), b.opacity);
    // V.A.T.S. on, or one of the game's menus open: the HUD shows only what
    // its mask leaves (`00771700`; the menus' masks by the menu that opened
    // from the game, `ui::hud::parts_for_menu`).
    let (parts, dialogue) = from.parts();
    let menu_mask = (parts != ui::hud::part::ALL).then_some(parts);
    match vats.hud_mask.or(menu_mask) {
        Some(bits) => b.hud.apply_mask(&mut b.ui, bits, &mut b.masked),
        None => b.hud.lift_mask(&mut b.ui, &mut b.masked),
    }
    b.hud.place_xp_meter(&mut b.ui, dialogue);
    let shown = if from.shown() { 1.0 } else { 0.0 };
    if b.ui.number(b.hud.menu, t::VISIBLE) != shown {
        b.ui.set_number(b.hud.menu, t::VISIBLE, shown);
    }
    b.ui.refresh();
    // V.A.T.S.'s menu, over the HUD, with the HUD's compass input.
    let vats_menu = match (b.vats.as_mut(), vats.menu.as_ref()) {
        (Some(menu), Some(wanted)) => {
            let mut wanted = wanted.clone();
            wanted.compass = ui::compass::CompassInput {
                heading: input.heading,
                position: input.position,
                interior: input.interior,
                markers: input.markers.clone(),
                actors: input.actors.clone(),
                opacity: input.opacity * 255.0,
            };
            menu.update(&mut b.ui, &wanted);
            Some((menu.menu, menu.tiles.compass.window, menu.compass_scroll))
        }
        _ => None,
    };

    let compass = b.hud.tiles.compass;
    let scroll = b.hud.compass_scroll;
    let mut items = {
        let mut files = Files {
            game,
            sizes: &mut b.sizes,
            atlases: &mut b.atlases,
        };
        let mut items = ui::draw_list(&mut b.ui, b.hud.menu, &mut files, &|tile| {
            (tile == compass).then(|| (scroll, ui::hud::COMPASS_ALPHA_MAP.to_string()))
        });
        if let Some((menu, window, vats_scroll)) = vats_menu {
            items.extend(ui::draw_list(&mut b.ui, menu, &mut files, &|tile| {
                (tile == window).then(|| (vats_scroll, ui::hud::COMPASS_ALPHA_MAP.to_string()))
            }));
        }
        items
    };
    // The game's menus over the HUD (their tiles are numbered in their own
    // tree; only the pictures are drawn here).
    items.extend(from.game_menus.0.iter().cloned());
    if items == b.last {
        return;
    }

    // Something changed: the pieces again.
    for (e, mesh, material) in b.drawn.drain(..) {
        commands.entity(e).despawn();
        meshes.remove(&mesh);
        materials.remove(&material);
    }
    let compressed = device
        .as_ref()
        .is_none_or(|d| d.features().contains(WgpuFeatures::TEXTURE_COMPRESSION_BC));
    let k = 1.0 / b.ui.screen_size.resolution_converter();
    for (i, item) in items.iter().enumerate() {
        let tint = Vec4::from_array(item.color);
        let mut pieces: Vec<Piece> = Vec::new();
        match &item.kind {
            DrawKind::Image {
                texture,
                rect,
                uv,
                repeat_u,
                scroll,
            } => {
                // `TILE1001` addresses its texture repeating both ways;
                // others repeat across when tiled, else clamp.
                let repeat = if scroll.is_some() {
                    (true, true)
                } else {
                    (*repeat_u, false)
                };
                let key = (texture.clone(), repeat.0, repeat.1);
                let handle = b
                    .images
                    .entry(key)
                    .or_insert_with(|| {
                        upload_picture(&mut images, game, texture, repeat, compressed)
                    })
                    .clone();
                let Some(handle) = handle else {
                    continue;
                };
                let (alpha_map, s) = match scroll {
                    Some((s, map)) => {
                        let key = (map.clone(), false, false);
                        let map = b
                            .images
                            .entry(key)
                            .or_insert_with(|| {
                                upload_picture(&mut images, game, map, (false, false), compressed)
                            })
                            .clone();
                        (map, *s)
                    }
                    None => (None, [0.0, 0.0, 1.0, 1.0]),
                };
                let corners = [
                    [uv[0], uv[1]],
                    [uv[2], uv[1]],
                    [uv[0], uv[3]],
                    [uv[2], uv[3]],
                ];
                pieces.push(Piece {
                    texture: handle,
                    alpha_map,
                    scroll: s,
                    quads: vec![(*rect, corners)],
                });
            }
            DrawKind::Text { font, glyphs } => {
                let Some(f) = b.ui.fonts.get(font - 1).cloned().flatten() else {
                    continue;
                };
                let paths = ui::draw::font_textures(&f);
                let mut by_picture: HashMap<u32, Vec<Quad>> = HashMap::new();
                for (rect, uv, picture) in glyphs {
                    by_picture.entry(*picture).or_default().push((*rect, *uv));
                }
                let mut pictures: Vec<_> = by_picture.into_iter().collect();
                pictures.sort_by_key(|(p, _)| *p);
                for (picture, quads) in pictures {
                    let Some(path) = paths.get(picture as usize) else {
                        continue;
                    };
                    let handle = b
                        .font_images
                        .entry((*font, picture))
                        .or_insert_with(|| upload_font_picture(&mut images, game, path))
                        .clone();
                    if let Some(texture) = handle {
                        pieces.push(Piece {
                            texture,
                            alpha_map: None,
                            scroll: [0.0, 0.0, 1.0, 1.0],
                            quads,
                        });
                    }
                }
            }
        }
        for Piece {
            texture,
            alpha_map,
            scroll,
            quads,
        } in pieces
        {
            let mesh = meshes.add(quads_mesh(&quads, k, size));
            let material = materials.add(TileMaterial {
                params: TileParams {
                    tint,
                    scroll: Vec4::from_array(scroll),
                    mode: Vec4::new(if alpha_map.is_some() { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0),
                },
                texture,
                alpha_map: alpha_map.unwrap_or_else(|| white.clone()),
            });
            // Back to front in the list's order.
            let entity = commands
                .spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(material.clone()),
                    Transform::from_xyz(0.0, 0.0, i as f32 * 0.01),
                    RenderLayers::layer(HUD_LAYER),
                ))
                .id();
            b.drawn.push((entity, mesh, material));
        }
    }
    b.last = items;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_activation_resolves_reference_to_acti_base_name() {
        use esm::{ActivePlugins, LoadOrder};
        use testdata::functions::ids::{CHEST_REF, CUP_REF, HOUSE, VIGOR_TESTER_REF};

        let data = testdata::functions::functions("hud-info-activation");
        let order = LoadOrder::from_data_dir(data.path(), &ActivePlugins::OfficialOnly).unwrap();
        let reference = esm::FormId(VIGOR_TESTER_REF);
        let base = world::scripting::base_of(&order, reference).unwrap();
        assert_ne!(base, reference);
        assert_eq!(
            order.get(base).unwrap().entry.header.kind,
            esm::FourCC::new(b"ACTI")
        );
        assert_eq!(
            supported_activation(&order, reference),
            Some((base, "sTargetTypeActivate", "Activate"))
        );
        let target = order
            .get(base)
            .unwrap()
            .record()
            .unwrap()
            .full_name()
            .unwrap();
        assert_eq!(target, "Vit-o-matic Vigor Tester");
        assert!(order.get(esm::FormId(HOUSE)).is_some());
        assert_eq!(
            supported_activation(&order, esm::FormId(CUP_REF))
                .unwrap()
                .1,
            "sTargetTypeTake"
        );
        assert_eq!(
            supported_activation(&order, esm::FormId(CHEST_REF))
                .unwrap()
                .1,
            "sTargetTypeOpen"
        );
    }

    #[test]
    fn quads_land_on_the_games_pixels() {
        // The HP meter at 60, 844 units, 296 x 20, on a 1920 x 1080 screen
        // (1.125 pixels a unit): pixels 68 to 401 across, 950 to 972.5 down,
        // half a pixel on from the game's numbers.
        let mesh = quads_mesh(
            &[(
                [60.0, 844.0, 296.0, 20.0],
                [[0.0, 0.0], [37.0, 0.0], [0.0, 0.625], [37.0, 0.625]],
            )],
            1.125,
            UVec2::new(1920, 1080),
        );
        let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(p)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("no positions");
        };
        // Top-left: 60 × 1.125 + 0.5 = 68 from the left; 844 × 1.125 + 0.5
        // = 950 from the top.
        assert_eq!(p[0], [68.0 - 960.0, 540.0 - 950.0, 0.0]);
        // Bottom-right.
        assert_eq!(
            p[3],
            [
                (356.0 * 1.125 + 0.5) - 960.0,
                540.0 - (864.0 * 1.125 + 0.5),
                0.0
            ]
        );
    }
}
