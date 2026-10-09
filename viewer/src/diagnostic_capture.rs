//! Presentation-side capture. CPU durations are not GPU timings.
use bevy::prelude::*;
use bevy::render::renderer::RenderAdapter;
use diagnostics::{Observer, Recorder, Value};
use std::time::{Duration, Instant};

pub fn start(args: &crate::args::Args, raw: &[String]) -> Option<Recorder> {
    if !args.diagnostics {
        return None;
    }
    match Recorder::start(
        &args.diagnostics_dir,
        &[
            ("build_revision", env!("NV_BUILD_REVISION").into()),
            ("build_dirty", Value::Bool(env!("NV_BUILD_DIRTY") == "true")),
            ("launch_arguments", format!("{raw:?}").into()),
            ("bevy", "0.16.1".into()),
            ("gpu_timing", "unavailable".into()),
            (
                "sync_render",
                Value::Bool(std::env::var_os("NV_SYNC_RENDER").is_some()),
            ),
        ],
    ) {
        Ok(r) => {
            let o = r.observer();
            o.install();
            println!("Diagnostics: {}", o.path().unwrap().display());
            o.emit("phase", o.id(), 0, &[("name", "startup".into())]);
            Some(r)
        }
        Err(e) => {
            eprintln!("Diagnostics unavailable: {e}; continuing play.");
            None
        }
    }
}
#[derive(Resource)]
struct Capture {
    observer: Observer,
    last: Instant,
    gameplay: bool,
    configured: bool,
    dropped: u64,
}
pub struct CapturePlugin;
impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let observer = Observer::current();
        if !observer.enabled() {
            return;
        }
        app.insert_resource(Capture {
            observer,
            last: Instant::now() - Duration::from_secs(1),
            gameplay: false,
            configured: false,
            dropped: 0,
        })
        .add_systems(Last, sample);
    }
}
#[allow(clippy::too_many_arguments)]
fn sample(
    mut capture: ResMut<Capture>,
    player: Res<crate::walk::Player>,
    state: Res<crate::dialogue::DialogueState>,
    cameras: Query<&Transform, With<crate::FlyCamera>>,
    windows: Query<&Window>,
    game: Res<crate::GameFiles>,
    exterior: Option<Res<crate::exterior::Exterior>>,
    adapter: Option<Res<RenderAdapter>>,
) {
    if capture.last.elapsed() < Duration::from_secs(1) {
        return;
    }
    capture.last = Instant::now();
    let o = capture.observer.clone();
    if !capture.configured {
        let plugins = game
            .0
            .order
            .plugins()
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join("|");
        let archives = game
            .0
            .assets
            .by_priority()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join("|");
        o.emit(
            "configuration",
            o.id(),
            0,
            &[
                ("plugin_order", plugins.into()),
                ("archive_order", archives.into()),
            ],
        );
        capture.configured = true;
    }
    if let Some(adapter) = adapter {
        let info = adapter.get_info();
        o.emit(
            "configuration",
            o.id(),
            0,
            &[
                ("adapter", info.name.into()),
                ("backend", format!("{:?}", info.backend).into()),
            ],
        );
    }
    if player.ready {
        capture.gameplay = true;
    }
    // Repeat the current phase in each sample: a dropped transition must not
    // classify the remainder of a session as startup.
    o.emit(
        "phase",
        o.id(),
        0,
        &[(
            "name",
            if capture.gameplay {
                "gameplay"
            } else {
                "startup"
            }
            .into(),
        )],
    );
    if let (Ok(camera), Ok(window)) = (cameras.single(), windows.single()) {
        let feet = player.position_for_view(crate::walk::game_point(camera.translation));
        o.emit(
            "player_sample",
            o.id(),
            0,
            &[
                ("x", Value::F64(feet[0] as f64)),
                ("y", Value::F64(feet[1] as f64)),
                ("z", Value::F64(feet[2] as f64)),
                ("cell", format!("{:?}", state.0.player_cell).into()),
                ("worldspace", format!("{:?}", state.0.player_world).into()),
                ("focused", Value::Bool(window.focused)),
                (
                    "loading",
                    Value::Bool(!player.ready || exterior.as_ref().is_some_and(|e| e.busy())),
                ),
                ("dropped_events", Value::U64(o.dropped())),
            ],
        );
        o.emit(
            "configuration",
            o.id(),
            0,
            &[
                ("width", Value::U64(window.physical_width() as u64)),
                ("height", Value::U64(window.physical_height() as u64)),
                ("present_mode", format!("{:?}", window.present_mode).into()),
            ],
        );
    }
    let dropped = o.dropped();
    if dropped != capture.dropped {
        eprintln!("Diagnostics: {dropped} events dropped.");
        capture.dropped = dropped;
    }
}
