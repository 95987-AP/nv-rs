//! `--fps`'s work times: how long each frame's own work takes on the main
//! thread (`First` to `Last`) and on the render thread (the render
//! schedule, extraction to cleanup), apart from any wait for the display.
//! A window behind others is held to the display's pace by Windows, so its
//! frame rate says little; its work still shows the hitches.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSet};

/// Work times gathered since the last report, in milliseconds.
#[derive(Default, Clone, Copy)]
pub struct Times {
    pub sum: f64,
    pub longest: f64,
    pub frames: u32,
}

impl Times {
    fn add(&mut self, ms: f64) {
        self.sum += ms;
        self.longest = self.longest.max(ms);
        self.frames += 1;
    }

    /// The mean and the longest, and starts again.
    pub fn take(&mut self) -> (f64, f64) {
        let mean = if self.frames == 0 {
            0.0
        } else {
            self.sum / f64::from(self.frames)
        };
        let out = (mean, self.longest);
        *self = Times::default();
        out
    }
}

/// Shared by both worlds: the main thread's and the render thread's times.
#[derive(Resource, Clone, Default)]
pub struct FrameWork {
    pub main: Arc<Mutex<Times>>,
    pub render: Arc<Mutex<Times>>,
}

#[derive(Resource, Default)]
struct MainStart(Option<Instant>, Option<Instant>, u64);

#[derive(Resource, Default)]
struct RenderStart(Option<Instant>, u64);

pub struct FrameWorkPlugin;

impl Plugin for FrameWorkPlugin {
    fn build(&self, app: &mut App) {
        let work = FrameWork::default();
        app.insert_resource(work.clone())
            .init_resource::<MainStart>()
            .add_systems(First, |mut start: ResMut<MainStart>| {
                let now = Instant::now();
                let o = diagnostics::Observer::current();
                start.2 = o.id();
                if o.enabled() {
                    if let Some(previous) = start.1 {
                        let ms = now.duration_since(previous).as_secs_f64() * 1000.0;
                        o.emit(
                            "main_frame_interval",
                            start.2,
                            0,
                            &[("interval_ms", diagnostics::Value::F64(ms))],
                        );
                        if ms > 50.0 {
                            o.emit(
                                "marker",
                                o.id(),
                                start.2,
                                &[
                                    ("reason", "slow_frame".into()),
                                    ("interval_ms", diagnostics::Value::F64(ms)),
                                ],
                            );
                        }
                    }
                    start.1 = Some(now);
                }
                start.0 = Some(now);
            })
            .add_systems(Last, |start: Res<MainStart>, work: Res<FrameWork>| {
                if let (Some(t), Ok(mut m)) = (start.0, work.main.lock()) {
                    let ms = t.elapsed().as_secs_f64() * 1000.0;
                    m.add(ms);
                    diagnostics::Observer::current().emit(
                        "main_frame",
                        start.2,
                        0,
                        &[("cpu_ms", diagnostics::Value::F64(ms))],
                    );
                }
            });
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .insert_resource(work)
                .init_resource::<RenderStart>()
                .add_systems(
                    Render,
                    (|mut start: ResMut<RenderStart>| {
                        start.0 = Some(Instant::now());
                        start.1 = diagnostics::Observer::current().id();
                    })
                    .in_set(RenderSet::ExtractCommands),
                )
                .add_systems(
                    Render,
                    (|start: Res<RenderStart>, work: Res<FrameWork>| {
                        if let (Some(t), Ok(mut r)) = (start.0, work.render.lock()) {
                            let ms = t.elapsed().as_secs_f64() * 1000.0;
                            r.add(ms);
                            diagnostics::Observer::current().emit(
                                "render_frame",
                                start.1,
                                0,
                                &[("cpu_ms", diagnostics::Value::F64(ms))],
                            );
                        }
                    })
                    .in_set(RenderSet::Cleanup),
                );
        }
    }
}
