//! Power-ups: drops, timed effects, HUD data.
//! Nine kinds: Spread, Rapid, Shield, Bomb, Life, Magnet, Freeze, Pierce, Double.

use macroquad::prelude::*;

pub const MAX_LIVES: i32 = 5;

pub const SPREAD_SECS: f32 = 12.0;
pub const RAPID_SECS: f32 = 10.0;
pub const MAGNET_SECS: f32 = 12.0;
pub const FREEZE_SECS: f32 = 8.0;
pub const PIERCE_SECS: f32 = 12.0;
pub const DOUBLE_SECS: f32 = 15.0;

pub const DROP_CHANCE: f32 = 0.10;
pub const TANK_DROP_CHANCE: f32 = 0.18;
pub const PICKUP_FALL_SPEED: f32 = 95.0;
pub const PICKUP_LIFETIME: f32 = 9.0;
pub const COLLECT_RADIUS: f32 = 26.0;
pub const MAGNET_PULL_RADIUS: f32 = 260.0;
pub const MAGNET_ZAP_RADIUS: f32 = 110.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerKind {
    Spread, // triple/quad shot
    Rapid,  // faster fire
    Shield, // absorbs one hit
    Bomb,   // destroys everything on screen
    Life,   // +1 life (rare)
    Magnet, // pulls pickups, zaps nearby enemy bullets
    Freeze, // slows + disarms enemies
    Pierce, // bullets pass through enemies
    Double, // 2x score
}

impl PowerKind {
    pub const ALL: [PowerKind; 9] = [
        PowerKind::Spread,
        PowerKind::Rapid,
        PowerKind::Shield,
        PowerKind::Bomb,
        PowerKind::Life,
        PowerKind::Magnet,
        PowerKind::Freeze,
        PowerKind::Pierce,
        PowerKind::Double,
    ];

    pub fn letter(self) -> char {
        match self {
            PowerKind::Spread => 'S',
            PowerKind::Rapid => 'R',
            PowerKind::Shield => 'H',
            PowerKind::Bomb => 'B',
            PowerKind::Life => '+',
            PowerKind::Magnet => 'M',
            PowerKind::Freeze => 'F',
            PowerKind::Pierce => 'P',
            PowerKind::Double => '2',
        }
    }

    pub fn color(self) -> Color {
        match self {
            PowerKind::Spread => GREEN,
            PowerKind::Rapid => YELLOW,
            PowerKind::Shield => SKYBLUE,
            PowerKind::Bomb => RED,
            PowerKind::Life => PINK,
            PowerKind::Magnet => BLUE,
            PowerKind::Freeze => WHITE,
            PowerKind::Pierce => ORANGE,
            PowerKind::Double => GOLD,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PowerKind::Spread => "SPREAD SHOT",
            PowerKind::Rapid => "RAPID FIRE",
            PowerKind::Shield => "SHIELD",
            PowerKind::Bomb => "BOMB",
            PowerKind::Life => "+1 LIFE",
            PowerKind::Magnet => "MAGNET",
            PowerKind::Freeze => "FREEZE",
            PowerKind::Pierce => "PIERCING",
            PowerKind::Double => "DOUBLE SCORE",
        }
    }

    /// Relative drop weight. Life only drops when it can help.
    fn weight(self, lives: i32) -> f32 {
        match self {
            PowerKind::Spread => 3.0,
            PowerKind::Rapid => 3.0,
            PowerKind::Double => 3.0,
            PowerKind::Pierce => 3.0,
            PowerKind::Shield => 2.0,
            PowerKind::Magnet => 2.0,
            PowerKind::Freeze => 2.0,
            PowerKind::Bomb => 1.2,
            PowerKind::Life => {
                if lives < MAX_LIVES {
                    0.8
                } else {
                    0.0
                }
            }
        }
    }

    /// Pure weighted pick on r in [0, total). Testable without RNG.
    pub fn pick_weighted(r: f32, lives: i32) -> PowerKind {
        let mut acc = 0.0;
        for k in Self::ALL {
            acc += k.weight(lives);
            if r < acc {
                return k;
            }
        }
        PowerKind::Spread
    }

    pub fn total_weight(lives: i32) -> f32 {
        Self::ALL.iter().map(|k| k.weight(lives)).sum()
    }

    /// Roll a drop for a killed enemy. `tank` doubles... no — caller passes
    /// the per-enemy chance and checks it first; this picks the kind.
    pub fn roll(lives: i32) -> PowerKind {
        let total = Self::total_weight(lives);
        Self::pick_weighted(rand::gen_range(0.0, total), lives)
    }
}

/// Active timed effects on the player.
#[derive(Clone, Debug)]
pub struct Effects {
    pub spread: f32,
    pub rapid: f32,
    pub shield: bool,
    pub magnet: f32,
    pub freeze: f32,
    pub pierce: f32,
    pub double: f32,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            spread: 0.0,
            rapid: 0.0,
            shield: false,
            magnet: 0.0,
            freeze: 0.0,
            pierce: 0.0,
            double: 0.0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn tick(&mut self, dt: f32) {
        self.spread = (self.spread - dt).max(0.0);
        self.rapid = (self.rapid - dt).max(0.0);
        self.magnet = (self.magnet - dt).max(0.0);
        self.freeze = (self.freeze - dt).max(0.0);
        self.pierce = (self.pierce - dt).max(0.0);
        self.double = (self.double - dt).max(0.0);
    }

    /// (letter, color, remaining, total) for HUD bars + shield pip.
    pub fn active(&self) -> Vec<(char, Color, f32, f32)> {
        let mut v = Vec::new();
        if self.spread > 0.0 {
            v.push(('S', GREEN, self.spread, SPREAD_SECS));
        }
        if self.rapid > 0.0 {
            v.push(('R', YELLOW, self.rapid, RAPID_SECS));
        }
        if self.magnet > 0.0 {
            v.push(('M', BLUE, self.magnet, MAGNET_SECS));
        }
        if self.freeze > 0.0 {
            v.push(('F', WHITE, self.freeze, FREEZE_SECS));
        }
        if self.pierce > 0.0 {
            v.push(('P', ORANGE, self.pierce, PIERCE_SECS));
        }
        if self.double > 0.0 {
            v.push(('2', GOLD, self.double, DOUBLE_SECS));
        }
        if self.shield {
            v.push(('H', SKYBLUE, 1.0, 1.0));
        }
        v
    }
}

pub struct Powerup {
    pub pos: Vec2,
    pub kind: PowerKind,
    pub t: f32,
    pub life: f32,
    pub dead: bool,
}

impl Powerup {
    pub fn new(pos: Vec2, kind: PowerKind) -> Self {
        Self {
            pos,
            kind,
            t: rand::gen_range(0.0, 6.28),
            life: 0.0,
            dead: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_nine_kinds_reachable() {
        let total = PowerKind::total_weight(3);
        assert!(total > 0.0);
        // sweep the whole range: every kind must be picked somewhere
        let mut seen = [false; 9];
        let steps = 2000;
        for i in 0..steps {
            let r = total * i as f32 / steps as f32;
            let k = PowerKind::pick_weighted(r, 3);
            seen[k as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "some kind unreachable: {seen:?}");
    }

    #[test]
    fn life_never_drops_at_max_lives() {
        let total = PowerKind::total_weight(MAX_LIVES);
        let steps = 2000;
        for i in 0..steps {
            let r = total * i as f32 / steps as f32;
            assert_ne!(PowerKind::pick_weighted(r, MAX_LIVES), PowerKind::Life);
        }
    }

    #[test]
    fn effects_tick_down_and_clear() {
        let mut fx = Effects::new();
        fx.spread = 5.0;
        fx.shield = true;
        fx.tick(2.0);
        assert_eq!(fx.spread, 3.0);
        assert!(fx.shield); // shield is until-hit, not timed
        fx.tick(10.0);
        assert_eq!(fx.spread, 0.0);
        fx.clear();
        assert!(fx.active().is_empty());
    }

    #[test]
    fn hud_lists_only_active_effects() {
        let mut fx = Effects::new();
        assert!(fx.active().is_empty());
        fx.double = 4.0;
        fx.shield = true;
        let a = fx.active();
        assert_eq!(a.len(), 2);
        assert!(a.iter().any(|&(c, _, _, _)| c == '2'));
        assert!(a.iter().any(|&(c, _, _, _)| c == 'H'));
    }

    #[test]
    fn letters_are_unique() {
        let mut letters: Vec<char> = PowerKind::ALL.iter().map(|k| k.letter()).collect();
        letters.sort();
        letters.dedup();
        assert_eq!(letters.len(), 9);
    }
}
