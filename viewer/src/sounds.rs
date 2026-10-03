//! The game's sounds (`world::sound`): doors opening, sounds scripts play
//! (`PlaySound`), and the loop the place plays (its acoustic space's, by
//! the hour). WAV files are decoded here (`cellview::sound`), OGG files by
//! Bevy. Sounds aren't placed in the world yet (no distance falloff or
//! direction), and music (MP3 files) isn't played.

use std::sync::Arc;
use std::time::Duration;

use bevy::audio::{AudioPlayer, AudioSource, Decodable, PlaybackSettings, Source};
use bevy::prelude::*;
use esm::FormId;
use world::sound::{ambient_loop, Sound};

use crate::dialogue::DialogueState;
use crate::GameFiles;

/// Decoded samples (from a WAV file).
#[derive(Asset, TypePath, Clone)]
pub struct PcmSound {
    channels: u16,
    rate: u32,
    samples: Arc<[i16]>,
}

pub struct PcmDecoder {
    sound: PcmSound,
    at: usize,
}

impl Iterator for PcmDecoder {
    type Item = i16;
    fn next(&mut self) -> Option<i16> {
        let s = self.sound.samples.get(self.at).copied();
        self.at += 1;
        s
    }
}

impl Source for PcmDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        self.sound.channels
    }
    fn sample_rate(&self) -> u32 {
        self.sound.rate
    }
    fn total_duration(&self) -> Option<Duration> {
        let frames = self.sound.samples.len() as f64 / f64::from(self.sound.channels.max(1));
        Some(Duration::from_secs_f64(
            frames / f64::from(self.sound.rate.max(1)),
        ))
    }
}

impl Decodable for PcmSound {
    type DecoderItem = i16;
    type Decoder = PcmDecoder;
    fn decoder(&self) -> PcmDecoder {
        PcmDecoder {
            sound: self.clone(),
            at: 0,
        }
    }
}

/// Sounds to play once (sound records), queued by doors and scripts.
#[derive(Resource, Default)]
pub struct SoundRequests(pub Vec<FormId>);

/// The place's loop playing now: for which (cell, sound), and its entity.
#[derive(Resource, Default)]
pub struct Ambient {
    key: Option<(FormId, FormId)>,
    entity: Option<Entity>,
}

/// Starts a sound record's file playing; the entity, or `None` when it
/// can't be found or read.
pub(crate) fn play(
    commands: &mut Commands,
    game: &cellview::Game,
    oggs: &mut Assets<AudioSource>,
    wavs: &mut Assets<PcmSound>,
    sound: &Sound,
    pick: u64,
    looping: bool,
) -> Option<Entity> {
    let (path, bytes) = game.sound_file(sound, pick)?;
    let settings = if looping {
        PlaybackSettings::LOOP
    } else {
        PlaybackSettings::DESPAWN
    };
    if path.ends_with(".ogg") {
        let handle = oggs.add(AudioSource {
            bytes: Arc::from(bytes.into_boxed_slice()),
        });
        return Some(commands.spawn((AudioPlayer::new(handle), settings)).id());
    }
    let pcm = cellview::sound::read_wav(&bytes)
        .map_err(|e| println!("  couldn't play {path}: {e}"))
        .ok()?;
    let handle = wavs.add(PcmSound {
        channels: pcm.channels,
        rate: pcm.rate,
        samples: Arc::from(pcm.samples.into_boxed_slice()),
    });
    Some(commands.spawn((AudioPlayer(handle), settings)).id())
}

/// Plays queued sounds, and keeps the place's loop going.
#[allow(clippy::too_many_arguments)]
pub fn play_sounds(
    mut commands: Commands,
    game: Res<GameFiles>,
    state: Res<DialogueState>,
    mut requests: ResMut<SoundRequests>,
    mut ambient: ResMut<Ambient>,
    mut oggs: ResMut<Assets<AudioSource>>,
    mut wavs: ResMut<Assets<PcmSound>>,
) {
    let order = &game.0.order;
    let state = &state.0;
    for (i, id) in std::mem::take(&mut requests.0).into_iter().enumerate() {
        if let Some(sound) = Sound::load(order, id) {
            let pick = state.dice.wrapping_add(i as u64);
            play(
                &mut commands,
                &game.0,
                &mut oggs,
                &mut wavs,
                &sound,
                pick,
                false,
            );
        }
    }
    // The place's loop: an interior's acoustic space, by the hour.
    let hour = state.global(order, "GameHour").unwrap_or(10.0);
    let wanted = match (state.player_world, state.player_cell) {
        (None, Some(cell)) => ambient_loop(order, cell, hour).map(|s| (cell, s)),
        _ => None,
    };
    if wanted == ambient.key {
        return;
    }
    if let Some(e) = ambient.entity.take() {
        if let Ok(mut entity) = commands.get_entity(e) {
            entity.despawn();
        }
    }
    ambient.key = wanted;
    if let Some((_, id)) = wanted {
        ambient.entity = Sound::load(order, id).and_then(|sound| {
            let entity = play(
                &mut commands,
                &game.0,
                &mut oggs,
                &mut wavs,
                &sound,
                state.dice,
                true,
            );
            match entity {
                Some(_) => println!("The place's sound: {}", sound.file),
                None => println!("The place's sound ({}) wasn't found.", sound.file),
            }
            entity
        });
    }
}
