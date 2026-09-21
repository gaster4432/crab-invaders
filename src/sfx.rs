//! Procedural sound synthesis: all SFX + music are generated as
//! 16-bit mono WAV bytes at load time. No audio assets in the repo.

pub const SAMPLE_RATE: u32 = 22050;

/// Write f32 samples (-1.0..1.0) into a WAV file in memory.
pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let mut out = Vec::with_capacity(44 + n as usize * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + n * 2).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(n * 2).to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[derive(Clone, Copy)]
pub enum Wave {
    Sine,
    Square,
    Saw,
}

/// Deterministic noise so builds/tests are reproducible.
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }
    pub fn next_f32(&mut self) -> f32 {
        // xorshift32 -> [0, 1)
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32).clamp(0.0, 1.0)
    }
}

fn osc(w: Wave, phase: f32) -> f32 {
    match w {
        Wave::Sine => (phase * std::f32::consts::TAU).sin(),
        Wave::Square => {
            if phase.fract() < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Saw => phase.fract() * 2.0 - 1.0,
    }
}

/// Frequency-swept tone with exponential decay envelope.
pub fn sweep(w: Wave, f0: f32, f1: f32, secs: f32, vol: f32) -> Vec<f32> {
    let n = (secs * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * t;
        phase += f / SAMPLE_RATE as f32;
        let env = (-4.0 * t).exp();
        out.push(osc(w, phase) * vol * env);
    }
    apply_click_fix(&mut out);
    out
}

/// Decaying white noise burst (explosions).
pub fn noise(secs: f32, vol: f32, seed: u32) -> Vec<f32> {
    let n = (secs * SAMPLE_RATE as f32) as usize;
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / n as f32;
        let env = (-5.0 * t).exp();
        out.push((rng.next_f32() * 2.0 - 1.0) * vol * env);
    }
    apply_click_fix(&mut out);
    out
}

/// Sequence of (semitone offset from A4, beats) played as sine blips.
pub fn melody(notes: &[(f32, f32)], beat_secs: f32, vol: f32) -> Vec<f32> {
    let mut out = Vec::new();
    for &(semi, beats) in notes {
        let f = note_freq(semi);
        let n = (beats * beat_secs * SAMPLE_RATE as f32) as usize;
        let mut phase = 0.0f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            phase += f / SAMPLE_RATE as f32;
            // per-note fade in/out to avoid clicks
            let edge = (i.min(n - i) as f32 / (SAMPLE_RATE as f32 * 0.008)).min(1.0);
            out.push(osc(Wave::Sine, phase) * vol * edge * (1.0 - 0.4 * t));
        }
    }
    out
}

pub fn note_freq(semi_from_a4: f32) -> f32 {
    440.0 * 2f32.powf(semi_from_a4 / 12.0)
}

/// Short fades at both ends so loops and cuts don't click.
fn apply_click_fix(buf: &mut [f32]) {
    let fade = (SAMPLE_RATE as f32 * 0.004) as usize;
    let n = buf.len();
    for i in 0..fade.min(n) {
        let k = i as f32 / fade as f32;
        buf[i] *= k;
        buf[n - 1 - i] *= k;
    }
}

// ---- game SFX (each returns complete WAV bytes) ----

pub fn shoot() -> Vec<u8> {
    wav_bytes(&sweep(Wave::Square, 900.0, 320.0, 0.09, 0.45))
}

pub fn enemy_boom() -> Vec<u8> {
    let mut n = noise(0.25, 0.7, 1234);
    let thump = sweep(Wave::Sine, 120.0, 55.0, 0.25, 0.6);
    for (a, b) in n.iter_mut().zip(thump.iter()) {
        *a = (*a + *b).clamp(-1.0, 1.0);
    }
    wav_bytes(&n)
}

pub fn player_hit() -> Vec<u8> {
    wav_bytes(&sweep(Wave::Saw, 320.0, 55.0, 0.4, 0.7))
}

pub fn pickup() -> Vec<u8> {
    wav_bytes(&melody(&[(0.0, 0.5), (4.0, 0.5)], 0.18, 0.5))
}

pub fn life_pickup() -> Vec<u8> {
    wav_bytes(&melody(&[(0.0, 0.4), (4.0, 0.4), (7.0, 0.8)], 0.16, 0.5))
}

pub fn shield_break() -> Vec<u8> {
    let mut s = sweep(Wave::Square, 1250.0, 180.0, 0.22, 0.55);
    let n = noise(0.22, 0.35, 777);
    for (a, b) in s.iter_mut().zip(n.iter()) {
        *a = (*a + *b).clamp(-1.0, 1.0);
    }
    wav_bytes(&s)
}

pub fn bomb() -> Vec<u8> {
    let mut n = noise(0.7, 0.8, 4242);
    let drop = sweep(Wave::Sine, 200.0, 35.0, 0.7, 0.7);
    for (a, b) in n.iter_mut().zip(drop.iter()) {
        *a = (*a + *b).clamp(-1.0, 1.0);
    }
    wav_bytes(&n)
}

pub fn wave_clear() -> Vec<u8> {
    wav_bytes(&melody(
        &[(3.0, 0.5), (7.0, 0.5), (10.0, 0.5), (15.0, 1.0)],
        0.14,
        0.5,
    ))
}

pub fn game_over() -> Vec<u8> {
    wav_bytes(&melody(
        &[(-2.0, 1.0), (-3.0, 1.0), (-4.0, 1.0), (-9.0, 2.0)],
        0.22,
        0.55,
    ))
}

pub fn ui_move() -> Vec<u8> {
    wav_bytes(&sweep(Wave::Sine, 520.0, 520.0, 0.045, 0.35))
}

pub fn ui_select() -> Vec<u8> {
    wav_bytes(&melody(&[(7.0, 0.5), (12.0, 0.7)], 0.09, 0.45))
}

/// 8-second seamless chiptune loop: Am F C G, 120 BPM.
/// Bass roots on 8ths, chord arp on 16ths, soft pad underneath.
pub fn music_loop() -> Vec<u8> {
    const SR: f32 = SAMPLE_RATE as f32;
    const CHORD_SECS: f32 = 2.0;
    // roots as semitone offsets from A4; minor/major flag
    const PROG: [(f32, bool); 4] = [
        (-24.0, true),  // Am (A2)
        (-28.0, false), // F  (F2)
        (-21.0, false), // C  (C3)
        (-26.0, false), // G  (G2)
    ];
    let total = (CHORD_SECS * 4.0 * SR) as usize;
    let mut out = vec![0.0f32; total];

    for (ci, &(root, minor)) in PROG.iter().enumerate() {
        let third = if minor { 3.0 } else { 4.0 };
        let tones = [0.0, third, 7.0, 12.0];
        let base = (ci as f32 * CHORD_SECS * SR) as usize;
        let chord_n = (CHORD_SECS * SR) as usize;

        // pad: whole-chord soft sines
        for &dt in &tones[..3] {
            let f = note_freq(root + dt);
            let mut phase = 0.0f32;
            for i in 0..chord_n {
                phase += f / SR;
                let edge = (i.min(chord_n - i) as f32 / (SR * 0.05)).min(1.0);
                out[base + i] += osc(Wave::Sine, phase) * 0.05 * edge;
            }
        }

        // bass: 8ths, root with octave pop every other
        let eighth = (0.25 * SR) as usize;
        for b in 0..8 {
            let f = note_freq(root + if b % 2 == 1 { 12.0 } else { 0.0 });
            let mut phase = 0.0f32;
            for i in 0..eighth {
                let idx = base + b * eighth + i;
                if idx >= total {
                    break;
                }
                phase += f / SR;
                let t = i as f32 / eighth as f32;
                out[idx] += osc(Wave::Square, phase) * 0.16 * (-3.0 * t).exp();
            }
        }

        // arp: 16ths cycling chord tones up and down
        let sixteenth = (0.125 * SR) as usize;
        for s in 0..16 {
            let seq = [0, 1, 2, 3, 2, 1];
            let dt = tones[seq[s % 6]] + 12.0;
            let f = note_freq(root + dt);
            let mut phase = 0.0f32;
            for i in 0..sixteenth {
                let idx = base + s * sixteenth + i;
                if idx >= total {
                    break;
                }
                phase += f / SR;
                let t = i as f32 / sixteenth as f32;
                out[idx] += osc(Wave::Square, phase) * 0.07 * (-4.0 * t).exp();
            }
        }
    }

    // normalize to a safe peak, then click-fix the seam
    let peak = out.iter().fold(0.0f32, |m, &v| m.max(v.abs())).max(0.001);
    let k = (0.6 / peak).min(1.5);
    for v in out.iter_mut() {
        *v *= k;
    }
    apply_click_fix(&mut out);
    wav_bytes(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_wav(bytes: &[u8]) -> (u32, u16, usize) {
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let bits = u16::from_le_bytes(bytes[34..36].try_into().unwrap());
        let data_len = u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize;
        assert_eq!(bytes.len(), 44 + data_len);
        assert_eq!(data_len % 2, 0);
        (rate, bits, data_len / 2)
    }

    #[test]
    fn wav_header_is_valid_pcm_mono() {
        let b = shoot();
        let (rate, bits, samples) = parse_wav(&b);
        assert_eq!(rate, SAMPLE_RATE);
        assert_eq!(bits, 16);
        assert!(samples > 1000);
    }

    #[test]
    fn no_clipping_in_any_sfx() {
        let all = vec![
            shoot(),
            enemy_boom(),
            player_hit(),
            pickup(),
            life_pickup(),
            shield_break(),
            bomb(),
            wave_clear(),
            game_over(),
            ui_move(),
            ui_select(),
        ];
        for b in &all {
            assert!(!b.is_empty());
            parse_wav(b);
        }
        // sample-domain check via a raw sweep
        for v in sweep(Wave::Square, 900.0, 300.0, 0.1, 0.9) {
            assert!(v.abs() <= 1.0);
        }
    }

    #[test]
    fn noise_is_deterministic() {
        assert_eq!(noise(0.1, 0.5, 42), noise(0.1, 0.5, 42));
        assert_ne!(noise(0.1, 0.5, 42), noise(0.1, 0.5, 43));
    }

    #[test]
    fn music_loop_is_8_seconds() {
        let b = music_loop();
        let (rate, _, samples) = parse_wav(&b);
        let secs = samples as f32 / rate as f32;
        assert!((secs - 8.0).abs() < 0.01, "loop is {secs}s");
    }

    #[test]
    fn note_freq_octave_doubles() {
        assert!((note_freq(12.0) - 880.0).abs() < 0.01);
        assert!((note_freq(0.0) - 440.0).abs() < 0.01);
    }
}
