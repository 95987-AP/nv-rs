//! The casino games' 3D on screen: the slot machine (`cellview::slots`, run
//! by `game_menus::slots`). Drawn as its `Draw3DElements` (`007c19c0`)
//! draws it: its own camera (the lockpicking menu's, `007c2180`) over the
//! scene, the depth cleared, under the menus' pictures; here, as the
//! Caravan table (`caravan_table`), into the HUD's picture by a camera of
//! its own before the HUD's, which then lays its pictures over it.
//!
//! Each frame every piece is put where its model's pose has it, under the
//! menu's node (`cellview::slots::root_transform`), and the reels' faces
//! take the textures the menu swapped onto them.

use std::sync::Arc;

use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::camera::{CameraOutputMode, Exposure, RenderTarget};
use bevy::render::render_resource::WgpuFeatures;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use cellview::lockpick::MenuCamera;
use cellview::slots::SlotModels;
use cellview::space;

use crate::game_menus::slots::SlotsScreen;
use crate::game_menus::{GameMenus, OpenMenu};
use crate::hud::HudLayer;
use crate::lighting::GameLitMaterial;
use crate::GameFiles;

/// The render layer the casino games' pieces are drawn on.
const CASINO_LAYER: usize = 27;

/// A piece: its model (0 the machine, 1 to 3 the reels), its shape, where
/// its mesh is centred.
#[derive(Component)]
struct CasinoPiece {
    model: usize,
    shape: String,
    center: Vec3,
    material: Handle<GameLitMaterial>,
}

/// What's on screen.
struct Shown {
    models: Arc<SlotModels>,
    entities: Vec<Entity>,
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<GameLitMaterial>>,
    images: Vec<Handle<Image>>,
    /// The reels' textures by slot (the casino's seven, the blank).
    reel_images: Vec<Option<Handle<Image>>>,
    texture_changes: u64,
}

#[derive(Resource, Default)]
pub struct CasinoShown {
    shown: Option<Shown>,
    layer: Option<Handle<Image>>,
}

pub struct CasinoScenePlugin;

impl Plugin for CasinoScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CasinoShown>()
            .add_systems(Update, show_casino.after(crate::menus::run_menus));
    }
}

/// The open slot machine, if any.
fn slots(menus: &GameMenus) -> Option<&SlotsScreen> {
    menus.screen.as_deref()?.open.iter().find_map(|m| match m {
        OpenMenu::Slots(s) if !s.closed => Some(&**s),
        _ => None,
    })
}

/// A piece's Bevy transform: the model's shape where its pose has it,
/// under the menu's node, in the camera's space (x ahead, y up, z right).
fn piece_transform(models: &SlotModels, s: &SlotsScreen, piece: &CasinoPiece) -> Option<Transform> {
    let model = models.models.get(piece.model)?;
    let moved = s.poses.get(piece.model)?.shape_move(model, &piece.shape)?;
    let m = cellview::slots::root_transform().then_child(&moved);
    let matrix = cellview::column_major(&m);
    Some(Transform::from_matrix(
        Mat4::from_cols_array(&space::matrix(&matrix)) * Mat4::from_translation(piece.center),
    ))
}

#[allow(clippy::too_many_arguments)]
fn show_casino(
    mut commands: Commands,
    game: Res<GameFiles>,
    settings: Res<crate::Settings>,
    menus: Res<GameMenus>,
    mut shown: ResMut<CasinoShown>,
    hud_layer: Option<Res<HudLayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GameLitMaterial>>,
    mut images: ResMut<Assets<Image>>,
    device: Option<Res<RenderDevice>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut pieces: Query<(&CasinoPiece, &mut Transform, &mut Visibility)>,
) {
    let open = slots(&menus);
    let CasinoShown { shown, layer } = &mut *shown;
    let stale = match (&*shown, open) {
        (Some(s), Some(o)) => !Arc::ptr_eq(&s.models, &o.models),
        (Some(_), None) => true,
        _ => false,
    };
    if stale {
        if let Some(s) = shown.take() {
            for e in s.entities {
                commands.entity(e).despawn();
            }
            for m in s.meshes {
                meshes.remove(&m);
            }
            for m in s.materials {
                materials.remove(&m);
            }
            for i in s.images {
                images.remove(&i);
            }
        }
    }
    let Some(open) = open else {
        return;
    };
    let models = open.models.clone();
    if shown.is_none() {
        let size = windows
            .single()
            .map(|w| UVec2::new(w.physical_width(), w.physical_height()))
            .unwrap_or(UVec2::new(1920, 1080))
            .max(UVec2::ONE);
        let compressed = device
            .as_ref()
            .is_none_or(|d| d.features().contains(WgpuFeatures::TEXTURE_COMPRESSION_BC));
        let target = crate::lockpick::menu_layer(
            &mut commands,
            hud_layer.as_deref(),
            layer,
            &mut images,
            size,
        );
        // The camera (`007c2180`): at the node's origin looking along +x
        // with +y up, the lockpicking menu's frustum.
        let camera = MenuCamera::new(&game.0.settings, size.x, size.y);
        let camera = commands
            .spawn((
                Camera3d::default(),
                Camera {
                    target: RenderTarget::from(target),
                    // Before the HUD's camera (−4), which draws the menus'
                    // pictures over it.
                    order: -5,
                    hdr: true,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    output_mode: CameraOutputMode::Write {
                        blend_state: None,
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                    },
                    ..default()
                },
                Tonemapping::None,
                DebandDither::Disabled,
                Projection::from(PerspectiveProjection {
                    fov: camera.vertical_fov(),
                    near: camera.near * space::METERS_PER_UNIT,
                    far: 100.0,
                    ..default()
                }),
                Exposure {
                    ev100: crate::START_EV100,
                },
                // Camera +x (ahead) is Bevy's +x, +y (up) Bevy's −z
                // (`space`'s conversion).
                Transform::from_translation(Vec3::ZERO).looking_to(Vec3::X, Vec3::NEG_Z),
                RenderLayers::layer(CASINO_LAYER),
            ))
            .id();
        let upload = |images: &mut Assets<Image>, t: &cellview::TextureData| {
            crate::upload_texture(images, t, compressed, settings.anisotropy)
        };
        let textures: Vec<Option<Handle<Image>>> = models
            .scene
            .textures
            .iter()
            .map(|t| upload(&mut images, t))
            .collect();
        let reel_images: Vec<Option<Handle<Image>>> = models
            .reel_textures
            .iter()
            .map(|t| t.as_ref().and_then(|t| upload(&mut images, t)))
            .collect();
        let mut s = Shown {
            models: models.clone(),
            entities: vec![camera],
            meshes: Vec::new(),
            materials: Vec::new(),
            images: textures
                .iter()
                .chain(reel_images.iter())
                .flatten()
                .cloned()
                .collect(),
            reel_images,
            texture_changes: u64::MAX,
        };
        let lighting = crate::lockpick::menu_lighting(&models.lights_now(), settings.brightness);
        for draw in &models.scene.draws {
            let Some(model) = (draw.reference as usize).checked_sub(1) else {
                continue;
            };
            let data = &models.scene.meshes[draw.mesh];
            let center = crate::sort_center(data).unwrap_or([0.0; 3]);
            let mesh = meshes.add(crate::game_mesh_around(data, center));
            let material = materials.add(crate::lit_material(data, &textures, lighting));
            s.meshes.push(mesh.clone());
            s.materials.push(material.clone());
            let e = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                    RenderLayers::layer(CASINO_LAYER),
                    crate::shared_light::MenuLit,
                    CasinoPiece {
                        model,
                        shape: data.shape_name.clone(),
                        center: Vec3::from(center),
                        material,
                    },
                ))
                .id();
            s.entities.push(e);
        }
        *shown = Some(s);
        // The pieces are there from the next frame (the commands run after
        // this): placed and textured then.
        return;
    }
    let Some(s) = shown.as_mut() else {
        return;
    };
    let faces_changed = s.texture_changes != open.texture_changes;
    s.texture_changes = open.texture_changes;
    for (piece, mut transform, mut visibility) in &mut pieces {
        let wanted = piece_transform(&models, open, piece);
        let wanted_visibility = if wanted.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted_visibility {
            *visibility = wanted_visibility;
        }
        if let Some(t) = wanted {
            if *transform != t {
                *transform = t;
            }
        }
        if faces_changed {
            let key = (piece.model, piece.shape.to_ascii_lowercase());
            let image = open
                .faces
                .get(&key)
                .and_then(|&slot| s.reel_images.get(slot).cloned().flatten());
            if let (Some(h), Some(mat)) = (image, materials.get_mut(&piece.material)) {
                if mat.base.base_color_texture.as_ref() != Some(&h) {
                    mat.base.base_color_texture = Some(h);
                }
            }
        }
    }
}
