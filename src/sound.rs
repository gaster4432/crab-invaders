//! Loaded audio handles. Everything is `Option<Sound>` so the game
//! still runs (silently) if audio init fails on some machine.

use crate::sfx;
use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes};

pub struct Sfx {
    pub shoot: Option<Sound>,
    pub boom: Option<Sound>,
    pub hit: Option<Sound>,
    pub pickup: Option<Sound>,
    pub life: Option<Sound>,
    pub shield: Option<Sound>,
    pub bomb: Option<Sound>,
    pub wave: Option<Sound>,
    pub over: Option<Sound>,
    pub ui_move: Option<Sound>,
    pub ui_select: Option<Sound>,
    pub music: Option<Sound>,
}

impl Sfx {
    pub fn empty() -> Self {
        Self {
            shoot: None,
            boom: None,
            hit: None,
            pickup: None,
            life: None,
            shield: None,
            bomb: None,
            wave: None,
            over: None,
            ui_move: None,
            ui_select: None,
            music: None,
        }
    }

    async fn load_one(bytes: Vec<u8>) -> Option<Sound> {
        load_sound_from_bytes(&bytes).await.ok()
    }

    /// Load + decode every sound. Takes ~1s (music synth); called once.
    pub async fn load() -> Self {
        // Synthesize first (pure CPU), then hand to the audio backend.
        let music = Self::load_one(sfx::music_loop()).await;
        Self {
            shoot: Self::load_one(sfx::shoot()).await,
            boom: Self::load_one(sfx::enemy_boom()).await,
            hit: Self::load_one(sfx::player_hit()).await,
            pickup: Self::load_one(sfx::pickup()).await,
            life: Self::load_one(sfx::life_pickup()).await,
            shield: Self::load_one(sfx::shield_break()).await,
            bomb: Self::load_one(sfx::bomb()).await,
            wave: Self::load_one(sfx::wave_clear()).await,
            over: Self::load_one(sfx::game_over()).await,
            ui_move: Self::load_one(sfx::ui_move()).await,
            ui_select: Self::load_one(sfx::ui_select()).await,
            music,
        }
    }

    pub fn start_music(&self, volume: f32) {
        if let Some(m) = &self.music {
            macroquad::audio::play_sound(
                m,
                PlaySoundParams {
                    looped: true,
                    volume: volume * 0.5,
                },
            );
        }
    }

    pub fn stop_music(&self) {
        if let Some(m) = &self.music {
            macroquad::audio::stop_sound(m);
        }
    }

    pub fn set_music_volume(&self, volume: f32) {
        if let Some(m) = &self.music {
            macroquad::audio::set_sound_volume(m, volume * 0.5);
        }
    }
}

/// Play a one-shot scaled by master volume. Silent when muted/missing.
pub fn play(sound: &Option<Sound>, base_vol: f32, master: f32, muted: bool) {
    if muted {
        return;
    }
    if let Some(s) = sound {
        macroquad::audio::play_sound(
            s,
            PlaySoundParams {
                looped: false,
                volume: (base_vol * master).clamp(0.0, 1.0),
            },
        );
    }
}
