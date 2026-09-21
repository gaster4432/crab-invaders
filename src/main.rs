mod menu;
mod powerups;
mod save;
mod sfx;
mod sound;

use macroquad::prelude::*;
use menu::{MenuScreen, MenuState};
use powerups::{
    Effects, PowerKind, Powerup, COLLECT_RADIUS, DROP_CHANCE, MAGNET_PULL_RADIUS,
    MAGNET_ZAP_RADIUS, MAX_LIVES, PICKUP_FALL_SPEED, PICKUP_LIFETIME, TANK_DROP_CHANCE,
};
use save::Save;
use sound::Sfx;

const PLAYER_SPEED: f32 = 420.0;
const BULLET_SPEED: f32 = 620.0;
const ENEMY_BULLET_SPEED: f32 = 260.0;
const FIRE_COOLDOWN: f32 = 0.16;
const RAPID_COOLDOWN: f32 = 0.08;

#[derive(PartialEq, Clone, Copy)]
enum State {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, PartialEq)]
enum EnemyKind {
    Diver,  // chases player slowly, 1 hp
    Weaver, // sine movement, 1 hp, shoots
    Tank,   // slow, 3 hp, shoots often
}

struct Player {
    pos: Vec2,
    cooldown: f32,
    lives: i32,
    invincible: f32,
}

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    friendly: bool,
    piercing: bool,
    dead: bool,
}

struct Enemy {
    pos: Vec2,
    vel: Vec2,
    kind: EnemyKind,
    hp: i32,
    fire_timer: f32,
    t: f32,
    pierce_cd: f32,
    dead: bool,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    color: Color,
    size: f32,
}

struct Floater {
    pos: Vec2,
    text: String,
    color: Color,
    life: f32,
}

struct Star {
    pos: Vec2,
    speed: f32,
}

struct Game {
    state: State,
    player: Player,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    particles: Vec<Particle>,
    powerups: Vec<Powerup>,
    floaters: Vec<Floater>,
    stars: Vec<Star>,
    fx: Effects,
    menu: MenuState,
    pause_sel: usize,
    over_sel: usize,
    score: u32,
    hi_score: u32,
    wave: u32,
    spawn_timer: f32,
    shake: f32,
    flash: f32,
    muted: bool,
    volume: f32,
    sfx: Sfx,
    quit: bool,
}

const PAUSE_ITEMS: [&str; 3] = ["RESUME", "RESTART", "QUIT TO MENU"];
const OVER_ITEMS: [&str; 2] = ["RETRY", "MENU"];

fn window_conf() -> Conf {
    Conf {
        window_title: "Crab Invaders".to_owned(),
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        ..Default::default()
    }
}

fn spawn_explosion(particles: &mut Vec<Particle>, pos: Vec2, color: Color, n: usize) {
    for _ in 0..n {
        let a = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(60.0, 320.0);
        particles.push(Particle {
            pos,
            vel: vec2(a.cos() * sp, a.sin() * sp),
            life: 0.0,
            max_life: rand::gen_range(0.3, 0.8),
            color,
            size: rand::gen_range(2.0, 5.0),
        });
    }
}

/// Click handling shared by menu / pause / game-over lists.
fn clicked_index(n_items: usize, w: f32, h: f32) -> Option<usize> {
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        for i in 0..n_items {
            if menu::clicked_on(&menu::item_rect(i, w, h), mx, my) {
                return Some(i);
            }
        }
    }
    None
}

/// Hover-highlight shared by menu / pause / game-over lists.
fn hover_index(n_items: usize, w: f32, h: f32) -> Option<usize> {
    let (mx, my) = mouse_position();
    for i in 0..n_items {
        if menu::clicked_on(&menu::item_rect(i, w, h), mx, my) {
            return Some(i);
        }
    }
    None
}

impl Game {
    fn new() -> Self {
        let saved = Save::load();
        let mut stars = Vec::new();
        for _ in 0..120 {
            stars.push(Star {
                pos: vec2(rand::gen_range(0.0, 800.0), rand::gen_range(0.0, 600.0)),
                speed: rand::gen_range(20.0, 140.0),
            });
        }
        Self {
            state: State::Menu,
            player: Player {
                pos: vec2(400.0, 520.0),
                cooldown: 0.0,
                lives: 3,
                invincible: 0.0,
            },
            bullets: Vec::new(),
            enemies: Vec::new(),
            particles: Vec::new(),
            powerups: Vec::new(),
            floaters: Vec::new(),
            stars,
            fx: Effects::new(),
            menu: MenuState::new(),
            pause_sel: 0,
            over_sel: 0,
            score: 0,
            hi_score: saved.hi,
            wave: 1,
            spawn_timer: 0.0,
            shake: 0.0,
            flash: 0.0,
            muted: saved.muted,
            volume: saved.volume,
            sfx: Sfx::empty(),
            quit: false,
        }
    }

    // ---- audio + persistence helpers ----

    fn fx_play(&self, s: &Option<macroquad::audio::Sound>, vol: f32) {
        sound::play(s, vol, self.volume, self.muted);
    }

    fn persist(&self) {
        Save {
            hi: self.hi_score,
            muted: self.muted,
            volume: self.volume,
        }
        .save();
    }

    fn save_hi(&mut self) {
        if self.score > self.hi_score {
            self.hi_score = self.score;
            self.persist();
        }
    }

    fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.sfx.stop_music();
        } else {
            self.sfx.start_music(self.volume);
        }
        self.persist();
    }

    fn bump_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.sfx.set_music_volume(self.volume);
        self.persist();
    }

    fn reset(&mut self) {
        self.save_hi();
        self.player = Player {
            pos: vec2(screen_width() / 2.0, screen_height() - 80.0),
            cooldown: 0.0,
            lives: 3,
            invincible: 1.0,
        };
        self.bullets.clear();
        self.enemies.clear();
        self.particles.clear();
        self.powerups.clear();
        self.floaters.clear();
        self.fx.clear();
        self.score = 0;
        self.wave = 1;
        self.spawn_timer = 0.0;
        self.shake = 0.0;
        self.flash = 0.0;
        self.pause_sel = 0;
        self.over_sel = 0;
        self.state = State::Playing;
    }

    fn start_run(&mut self) {
        self.fx_play(&self.sfx.ui_select, 0.6);
        self.reset();
        self.spawn_wave();
        self.sfx.start_music(self.volume);
    }

    fn to_menu(&mut self) {
        self.save_hi();
        self.menu.goto(MenuScreen::Main);
        self.state = State::Menu;
        self.sfx.start_music(self.volume);
    }

    fn die(&mut self) {
        self.save_hi();
        self.sfx.stop_music();
        self.fx_play(&self.sfx.over, 0.7);
        self.state = State::GameOver;
    }

    fn spawn_wave(&mut self) {
        let w = screen_width();
        // Count grows with wave, capped for sanity
        let count = (4 + self.wave * 2).min(28) as usize;
        for i in 0..count {
            let kind = match rand::gen_range(0, 10) {
                0..=4 => EnemyKind::Diver,
                5..=7 => EnemyKind::Weaver,
                _ => EnemyKind::Tank,
            };
            let x = rand::gen_range(30.0, w - 30.0);
            let y = rand::gen_range(-140.0, -30.0 - i as f32 * 12.0);
            let (vel, hp, fire_timer) = match kind {
                EnemyKind::Diver => (
                    vec2(rand::gen_range(-40.0, 40.0), rand::gen_range(70.0, 110.0)),
                    1,
                    999.0,
                ),
                EnemyKind::Weaver => (
                    vec2(rand::gen_range(-90.0, 90.0), rand::gen_range(50.0, 90.0)),
                    1,
                    rand::gen_range(1.0, 2.5),
                ),
                EnemyKind::Tank => (
                    vec2(rand::gen_range(-25.0, 25.0), rand::gen_range(30.0, 55.0)),
                    3,
                    rand::gen_range(0.8, 1.8),
                ),
            };
            self.enemies.push(Enemy {
                pos: vec2(x, y),
                vel,
                kind,
                hp,
                fire_timer,
                t: rand::gen_range(0.0, 6.28),
                pierce_cd: 0.0,
                dead: false,
            });
        }
    }

    fn floater(&mut self, pos: Vec2, text: &str, color: Color) {
        self.floaters.push(Floater {
            pos,
            text: text.to_string(),
            color,
            life: 0.0,
        });
    }

    fn score_for(&self, base: u32) -> u32 {
        if self.fx.double > 0.0 {
            base * 2
        } else {
            base
        }
    }

    fn enemy_points(kind: EnemyKind) -> (u32, Color) {
        match kind {
            EnemyKind::Diver => (50, RED),
            EnemyKind::Weaver => (100, ORANGE),
            EnemyKind::Tank => (250, PURPLE),
        }
    }

    /// Bomb pickup: wipe the screen, score everything.
    fn detonate(&mut self) {
        let mut gained = 0u32;
        for e in self.enemies.iter_mut().filter(|e| !e.dead) {
            e.dead = true;
            let (pts, col) = Self::enemy_points(e.kind);
            gained += pts;
            spawn_explosion(&mut self.particles, e.pos, col, 14);
        }
        for b in self.bullets.iter_mut().filter(|b| !b.friendly) {
            b.dead = true;
        }
        self.score += self.score_for(gained);
        self.flash = 0.45;
        self.shake = 20.0;
        self.fx_play(&self.sfx.bomb, 0.9);
        let c = vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.floater(c, "BOOM!", YELLOW);
    }

    fn apply_powerup(&mut self, kind: PowerKind, at: Vec2) {
        match kind {
            PowerKind::Spread => {
                self.fx.spread = powerups::SPREAD_SECS;
                self.floater(at, "SPREAD SHOT", GREEN);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
            PowerKind::Rapid => {
                self.fx.rapid = powerups::RAPID_SECS;
                self.floater(at, "RAPID FIRE", YELLOW);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
            PowerKind::Shield => {
                self.fx.shield = true;
                self.floater(at, "SHIELD UP", SKYBLUE);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
            PowerKind::Bomb => self.detonate(),
            PowerKind::Life => {
                if self.player.lives < MAX_LIVES {
                    self.player.lives += 1;
                    self.floater(at, "+1 LIFE", PINK);
                } else {
                    let bonus = self.score_for(500);
                    self.score += bonus;
                    self.floater(at, "+500", PINK);
                }
                self.fx_play(&self.sfx.life, 0.7);
            }
            PowerKind::Magnet => {
                self.fx.magnet = powerups::MAGNET_SECS;
                self.floater(at, "MAGNET", BLUE);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
            PowerKind::Freeze => {
                self.fx.freeze = powerups::FREEZE_SECS;
                self.floater(at, "FREEZE", WHITE);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
            PowerKind::Pierce => {
                self.fx.pierce = powerups::PIERCE_SECS;
                self.floater(at, "PIERCING", ORANGE);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
            PowerKind::Double => {
                self.fx.double = powerups::DOUBLE_SECS;
                self.floater(at, "DOUBLE SCORE", GOLD);
                self.fx_play(&self.sfx.pickup, 0.6);
            }
        }
    }

    // ---- menus ----

    fn update_menu(&mut self) {
        self.update_stars();
        let w = screen_width();
        let h = screen_height();

        if is_key_pressed(KeyCode::M) {
            self.toggle_mute();
            return;
        }

        let n = self.menu.items(self.muted, self.volume).len();
        let mut moved = false;
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.menu.up(self.muted, self.volume);
            moved = true;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.menu.down(self.muted, self.volume);
            moved = true;
        }
        if let Some(i) = hover_index(n, w, h) {
            if i != self.menu.selected {
                self.menu.selected = i;
                moved = true;
            }
        }
        if moved {
            self.fx_play(&self.sfx.ui_move, 0.4);
        }

        if self.menu.screen == MenuScreen::Settings {
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                self.bump_volume(-0.1);
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                self.bump_volume(0.1);
            }
            if is_key_pressed(KeyCode::Escape) {
                self.fx_play(&self.sfx.ui_select, 0.5);
                self.menu.goto(MenuScreen::Main);
                return;
            }
        } else if is_key_pressed(KeyCode::Escape) {
            if self.menu.screen != MenuScreen::Main {
                self.menu.goto(MenuScreen::Main);
            }
            return;
        }

        let mut activate = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if let Some(i) = clicked_index(n, w, h) {
            self.menu.selected = i;
            activate = true;
        }
        if !activate {
            return;
        }

        match self.menu.screen {
            MenuScreen::Main => match self.menu.selected {
                0 => self.start_run(),
                1 => {
                    self.fx_play(&self.sfx.ui_select, 0.5);
                    self.menu.goto(MenuScreen::Help);
                }
                2 => {
                    self.fx_play(&self.sfx.ui_select, 0.5);
                    self.menu.goto(MenuScreen::Settings);
                }
                _ => {
                    self.fx_play(&self.sfx.ui_select, 0.5);
                    self.save_hi();
                    self.quit = true;
                }
            },
            MenuScreen::Help => {
                self.fx_play(&self.sfx.ui_select, 0.5);
                self.menu.goto(MenuScreen::Main);
            }
            MenuScreen::Settings => match self.menu.selected {
                0 => {
                    self.toggle_mute();
                    self.fx_play(&self.sfx.ui_select, 0.5);
                }
                1 => self.bump_volume(0.1),
                _ => {
                    self.fx_play(&self.sfx.ui_select, 0.5);
                    self.menu.goto(MenuScreen::Main);
                }
            },
        }
    }

    fn update_paused(&mut self) {
        self.update_stars();
        let w = screen_width();
        let h = screen_height();

        if is_key_pressed(KeyCode::M) {
            self.toggle_mute();
            return;
        }
        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            self.state = State::Playing;
            return;
        }
        if is_key_pressed(KeyCode::Q) {
            self.to_menu();
            return;
        }

        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.pause_sel = (self.pause_sel + PAUSE_ITEMS.len() - 1) % PAUSE_ITEMS.len();
            self.fx_play(&self.sfx.ui_move, 0.4);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.pause_sel = (self.pause_sel + 1) % PAUSE_ITEMS.len();
            self.fx_play(&self.sfx.ui_move, 0.4);
        }
        if let Some(i) = hover_index(PAUSE_ITEMS.len(), w, h) {
            if i != self.pause_sel {
                self.pause_sel = i;
                self.fx_play(&self.sfx.ui_move, 0.4);
            }
        }

        let mut activate = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if let Some(i) = clicked_index(PAUSE_ITEMS.len(), w, h) {
            self.pause_sel = i;
            activate = true;
        }
        if !activate {
            return;
        }
        self.fx_play(&self.sfx.ui_select, 0.5);
        match self.pause_sel {
            0 => self.state = State::Playing,
            1 => {
                self.reset();
                self.spawn_wave();
            }
            _ => self.to_menu(),
        }
    }

    fn update_gameover(&mut self) {
        self.update_stars();
        let dt = get_frame_time();
        for p in &mut self.particles {
            p.life += dt;
            p.pos += p.vel * dt;
        }
        self.particles.retain(|p| p.life < p.max_life);
        self.update_floaters(dt);

        let w = screen_width();
        let h = screen_height();

        if is_key_pressed(KeyCode::M) {
            self.toggle_mute();
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.to_menu();
            return;
        }

        if is_key_pressed(KeyCode::Up)
            || is_key_pressed(KeyCode::Down)
            || is_key_pressed(KeyCode::W)
            || is_key_pressed(KeyCode::S)
        {
            self.over_sel = (self.over_sel + 1) % OVER_ITEMS.len();
            self.fx_play(&self.sfx.ui_move, 0.4);
        }
        if let Some(i) = hover_index(OVER_ITEMS.len(), w, h) {
            if i != self.over_sel {
                self.over_sel = i;
                self.fx_play(&self.sfx.ui_move, 0.4);
            }
        }

        let mut activate = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if let Some(i) = clicked_index(OVER_ITEMS.len(), w, h) {
            self.over_sel = i;
            activate = true;
        }
        if !activate {
            return;
        }
        if self.over_sel == 0 {
            self.start_run();
        } else {
            self.to_menu();
        }
    }

    fn update_stars(&mut self) {
        let dt = get_frame_time();
        let h = screen_height();
        let w = screen_width();
        for s in &mut self.stars {
            s.pos.y += s.speed * dt;
            if s.pos.y > h {
                s.pos.y = -2.0;
                s.pos.x = rand::gen_range(0.0, w);
            }
        }
    }

    fn update_floaters(&mut self, dt: f32) {
        for f in &mut self.floaters {
            f.life += dt;
            f.pos.y -= 40.0 * dt;
        }
        self.floaters.retain(|f| f.life < 1.4);
    }

    // ---- gameplay ----

    fn fire_guns(&mut self) {
        let piercing = self.fx.pierce > 0.0;
        let mk = |pos: Vec2, vel: Vec2| Bullet {
            pos,
            vel,
            friendly: true,
            piercing,
            dead: false,
        };
        // twin parallel guns, always
        self.bullets.push(mk(
            self.player.pos + vec2(-8.0, -14.0),
            vec2(0.0, -BULLET_SPEED),
        ));
        self.bullets.push(mk(
            self.player.pos + vec2(8.0, -14.0),
            vec2(0.0, -BULLET_SPEED),
        ));
        // spread adds two angled shots
        if self.fx.spread > 0.0 {
            let a = 0.18f32;
            self.bullets.push(mk(
                self.player.pos + vec2(0.0, -14.0),
                vec2((a).sin() * BULLET_SPEED, -(a).cos() * BULLET_SPEED),
            ));
            self.bullets.push(mk(
                self.player.pos + vec2(0.0, -14.0),
                vec2((-a).sin() * BULLET_SPEED, -(a).cos() * BULLET_SPEED),
            ));
        }
        self.fx_play(&self.sfx.shoot, 0.22);
    }

    fn update_playing(&mut self) {
        let dt = get_frame_time();
        let w = screen_width();
        let h = screen_height();

        self.update_stars();

        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            self.pause_sel = 0;
            self.state = State::Paused;
            return;
        }
        if is_key_pressed(KeyCode::M) {
            self.toggle_mute();
        }

        self.fx.tick(dt);
        self.flash = (self.flash - dt).max(0.0);

        // --- player movement ---
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            dir.x += 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            dir.y += 1.0;
        }
        if dir.length_squared() > 0.0 {
            dir = dir.normalize();
        }
        self.player.pos += dir * PLAYER_SPEED * dt;
        self.player.pos.x = self.player.pos.x.clamp(20.0, w - 20.0);
        self.player.pos.y = self.player.pos.y.clamp(h * 0.35, h - 24.0);
        self.player.cooldown -= dt;
        self.player.invincible -= dt;

        if (is_key_down(KeyCode::Space) || is_key_down(KeyCode::J))
            && self.player.cooldown <= 0.0
        {
            self.player.cooldown = if self.fx.rapid > 0.0 {
                RAPID_COOLDOWN
            } else {
                FIRE_COOLDOWN
            };
            self.fire_guns();
        }

        // --- spawn next wave when cleared ---
        if self.enemies.is_empty() && self.powerups.is_empty() {
            self.spawn_timer += dt;
            if self.spawn_timer > 1.2 {
                self.wave += 1;
                self.spawn_timer = 0.0;
                self.spawn_wave();
                self.fx_play(&self.sfx.wave, 0.6);
                self.floater(
                    vec2(w / 2.0, h / 2.0 - 40.0),
                    &format!("WAVE {}", self.wave),
                    YELLOW,
                );
                // small heal every 5 waves
                if self.wave % 5 == 0 && self.player.lives < MAX_LIVES {
                    self.player.lives += 1;
                }
            }
        } else {
            self.spawn_timer = 0.0;
        }

        // --- enemies (slowed + disarmed while frozen) ---
        let frozen = self.fx.freeze > 0.0;
        let edt = if frozen { dt * 0.25 } else { dt };
        let player_pos = self.player.pos;
        let mut new_enemy_bullets: Vec<Bullet> = Vec::new();
        for e in &mut self.enemies {
            e.t += edt;
            e.pierce_cd = (e.pierce_cd - dt).max(0.0);
            match e.kind {
                EnemyKind::Diver => {
                    // drift toward player x
                    let dx = player_pos.x - e.pos.x;
                    e.pos.x += dx.clamp(-1.0, 1.0) * 60.0 * edt;
                    e.pos += e.vel * edt;
                }
                EnemyKind::Weaver => {
                    e.pos += e.vel * edt;
                    e.pos.x += (e.t * 3.0).sin() * 120.0 * edt;
                    if e.pos.x < 16.0 || e.pos.x > w - 16.0 {
                        e.vel.x = -e.vel.x;
                        e.pos.x = e.pos.x.clamp(16.0, w - 16.0);
                    }
                }
                EnemyKind::Tank => {
                    e.pos += e.vel * edt;
                }
            }
            // wrap / floor: enemies that pass bottom re-enter top
            if e.pos.y > h + 30.0 {
                e.pos.y = -30.0;
                e.pos.x = rand::gen_range(20.0, w - 20.0);
            }
            // shooting (not while frozen)
            if !frozen && e.kind != EnemyKind::Diver {
                e.fire_timer -= dt;
                if e.fire_timer <= 0.0 && e.pos.y > 0.0 && e.pos.y < h * 0.8 {
                    e.fire_timer = match e.kind {
                        EnemyKind::Weaver => rand::gen_range(1.4, 2.8),
                        EnemyKind::Tank => rand::gen_range(0.9, 1.9),
                        EnemyKind::Diver => 999.0,
                    };
                    let dir = (player_pos - e.pos).normalize_or_zero();
                    new_enemy_bullets.push(Bullet {
                        pos: e.pos + vec2(0.0, 12.0),
                        vel: dir * ENEMY_BULLET_SPEED,
                        friendly: false,
                        piercing: false,
                        dead: false,
                    });
                }
            }
        }
        self.bullets.extend(new_enemy_bullets);

        // --- bullets ---
        for b in &mut self.bullets {
            b.pos += b.vel * dt;
            if b.pos.y < -20.0 || b.pos.y > h + 20.0 || b.pos.x < -20.0 || b.pos.x > w + 20.0
            {
                b.dead = true;
            }
        }

        // friendly bullets vs enemies (index-based to satisfy borrow checker)
        let mut boom_events: Vec<(Vec2, Color, usize)> = Vec::new();
        let mut shake_add = 0.0;
        let mut score_add = 0u32;
        let mut drop_rolls: Vec<(Vec2, EnemyKind)> = Vec::new();
        let mut plays_boom = false;
        for bi in 0..self.bullets.len() {
            if !self.bullets[bi].friendly || self.bullets[bi].dead {
                continue;
            }
            let bpos = self.bullets[bi].pos;
            let piercing = self.bullets[bi].piercing;
            for ei in 0..self.enemies.len() {
                if self.enemies[ei].dead {
                    continue;
                }
                if piercing && self.enemies[ei].pierce_cd > 0.0 {
                    continue;
                }
                let r = if self.enemies[ei].kind == EnemyKind::Tank {
                    18.0
                } else {
                    13.0
                };
                if bpos.distance(self.enemies[ei].pos) < r + 4.0 {
                    if piercing {
                        self.enemies[ei].pierce_cd = 0.12;
                    } else {
                        self.bullets[bi].dead = true;
                    }
                    self.enemies[ei].hp -= 1;
                    boom_events.push((bpos, YELLOW, 4));
                    if self.enemies[ei].hp <= 0 {
                        self.enemies[ei].dead = true;
                        let (pts, col) = Self::enemy_points(self.enemies[ei].kind);
                        score_add += pts;
                        boom_events.push((self.enemies[ei].pos, col, 22));
                        shake_add += 6.0;
                        plays_boom = true;
                        drop_rolls.push((self.enemies[ei].pos, self.enemies[ei].kind));
                    }
                    if !piercing {
                        break;
                    }
                }
            }
        }
        self.score += self.score_for(score_add);
        self.shake = (self.shake + shake_add).min(12.0);
        for (pos, col, n) in boom_events {
            spawn_explosion(&mut self.particles, pos, col, n);
        }
        if plays_boom {
            self.fx_play(&self.sfx.boom, 0.45);
        }
        // power-up drops
        for (pos, kind) in drop_rolls {
            let chance = if kind == EnemyKind::Tank {
                TANK_DROP_CHANCE
            } else {
                DROP_CHANCE
            };
            if rand::gen_range(0.0, 1.0) < chance {
                self.powerups
                    .push(Powerup::new(pos, PowerKind::roll(self.player.lives)));
            }
        }

        // magnet zaps nearby enemy bullets
        if self.fx.magnet > 0.0 {
            let ppos = self.player.pos;
            for b in self.bullets.iter_mut().filter(|b| !b.friendly && !b.dead) {
                if b.pos.distance(ppos) < MAGNET_ZAP_RADIUS {
                    b.dead = true;
                    spawn_explosion(&mut self.particles, b.pos, BLUE, 3);
                }
            }
        }

        // enemy bullets / bodies vs player
        if self.player.invincible <= 0.0 {
            let mut hit = false;
            let ppos = self.player.pos;
            for b in self.bullets.iter_mut().filter(|b| !b.friendly && !b.dead) {
                if b.pos.distance(ppos) < 14.0 {
                    b.dead = true;
                    hit = true;
                    break;
                }
            }
            let mut ram_pos: Option<Vec2> = None;
            if !hit {
                for e in self.enemies.iter_mut() {
                    if !e.dead && e.pos.distance(ppos) < 22.0 {
                        e.dead = true;
                        ram_pos = Some(e.pos);
                        hit = true;
                        break;
                    }
                }
            }
            if let Some(pos) = ram_pos {
                spawn_explosion(&mut self.particles, pos, RED, 22);
            }
            if hit {
                if self.fx.shield {
                    // shield absorbs the hit
                    self.fx.shield = false;
                    self.player.invincible = 1.5;
                    self.fx_play(&self.sfx.shield, 0.7);
                    self.floater(self.player.pos + vec2(0.0, -30.0), "SHIELD DOWN", SKYBLUE);
                    self.shake = (self.shake + 8.0).min(12.0);
                } else {
                    self.player.lives -= 1;
                    self.player.invincible = 2.0;
                    let ppos = self.player.pos;
                    spawn_explosion(&mut self.particles, ppos, SKYBLUE, 30);
                    self.fx_play(&self.sfx.hit, 0.8);
                    self.shake = 14.0;
                    if self.player.lives <= 0 {
                        self.bullets.retain(|b| !b.dead);
                        self.enemies.retain(|e| !e.dead);
                        self.die();
                        return;
                    }
                }
            }
        }

        // --- power-up pickups ---
        let magnet = self.fx.magnet > 0.0;
        let ppos = self.player.pos;
        let mut collected: Vec<PowerKind> = Vec::new();
        for p in &mut self.powerups {
            p.t += dt;
            p.life += dt;
            p.pos.y += PICKUP_FALL_SPEED * dt;
            p.pos.x += (p.t * 2.5).sin() * 40.0 * dt;
            if magnet {
                let d = ppos - p.pos;
                if d.length() < MAGNET_PULL_RADIUS {
                    p.pos += d.normalize_or_zero() * 320.0 * dt;
                }
            }
            if p.life > PICKUP_LIFETIME || p.pos.y > h + 20.0 {
                p.dead = true;
                continue;
            }
            if p.pos.distance(ppos) < COLLECT_RADIUS {
                p.dead = true;
                collected.push(p.kind);
            }
        }
        for kind in collected {
            self.apply_powerup(kind, ppos + vec2(0.0, -24.0));
        }
        self.powerups.retain(|p| !p.dead);

        self.bullets.retain(|b| !b.dead);
        self.enemies.retain(|e| !e.dead);

        // --- particles + floaters ---
        for p in &mut self.particles {
            p.life += dt;
            p.pos += p.vel * dt;
            p.vel *= 1.0 - 2.2 * dt;
        }
        self.particles.retain(|p| p.life < p.max_life);
        self.update_floaters(dt);

        self.shake = (self.shake - dt * 30.0).max(0.0);
    }

    // ---- drawing ----

    fn draw(&self) {
        let w = screen_width();
        let h = screen_height();
        let ox = if self.shake > 0.0 {
            rand::gen_range(-self.shake, self.shake)
        } else {
            0.0
        };
        let oy = if self.shake > 0.0 {
            rand::gen_range(-self.shake, self.shake)
        } else {
            0.0
        };

        clear_background(Color::new(0.04, 0.05, 0.10, 1.0));

        // stars
        for s in &self.stars {
            draw_circle(s.pos.x, s.pos.y, 1.5, Color::new(1.0, 1.0, 1.0, 0.55));
        }

        if self.state == State::Menu {
            menu::draw_title(w, h, self.hi_score);
            match self.menu.screen {
                MenuScreen::Main => {
                    let items = self.menu.items(self.muted, self.volume);
                    menu::draw_items(w, h, &items, self.menu.selected);
                }
                MenuScreen::Help => {
                    menu::draw_help(w, h);
                    menu::draw_items(w, h, &["BACK".to_string()], self.menu.selected);
                }
                MenuScreen::Settings => {
                    let items = self.menu.items(self.muted, self.volume);
                    menu::draw_items(w, h, &items, self.menu.selected);
                    let hint = "LEFT/RIGHT adjusts volume";
                    let m = measure_text(hint, None, 16, 1.0);
                    draw_text(hint, w / 2.0 - m.width / 2.0, h - 48.0, 16.0, GRAY);
                }
            }
            return;
        }

        let frozen = self.fx.freeze > 0.0;

        // enemies
        for e in &self.enemies {
            let (col, r) = match e.kind {
                EnemyKind::Diver => (RED, 13.0),
                EnemyKind::Weaver => (ORANGE, 13.0),
                EnemyKind::Tank => (PURPLE, 18.0),
            };
            draw_circle(e.pos.x + ox, e.pos.y + oy, r, col);
            draw_circle(e.pos.x + ox, e.pos.y + oy, r - 5.0, DARKGRAY);
            draw_circle(e.pos.x + ox, e.pos.y + oy, 3.0, WHITE);
            if frozen {
                draw_circle(
                    e.pos.x + ox,
                    e.pos.y + oy,
                    r + 2.0,
                    Color::new(0.6, 0.85, 1.0, 0.35),
                );
            }
            if e.kind == EnemyKind::Tank {
                // hp pips
                for i in 0..e.hp {
                    draw_rectangle(
                        e.pos.x + ox - 12.0 + i as f32 * 9.0,
                        e.pos.y + oy - r - 10.0,
                        7.0,
                        4.0,
                        GREEN,
                    );
                }
            }
        }

        // power-up pickups (blink when about to expire)
        for p in &self.powerups {
            let blink = p.life > PICKUP_LIFETIME - 2.0 && (p.t * 10.0).fract() < 0.5;
            if blink {
                continue;
            }
            let x = p.pos.x + ox;
            let y = p.pos.y + oy;
            draw_circle(x, y, 12.0, p.kind.color());
            draw_circle(x, y, 9.0, BLACK);
            let letter = p.kind.letter().to_string();
            let m = measure_text(&letter, None, 18, 1.0);
            draw_text(&letter, x - m.width / 2.0, y + 6.0, 18.0, p.kind.color());
        }

        // bullets
        for b in &self.bullets {
            if b.friendly {
                let c = if b.piercing { ORANGE } else { YELLOW };
                draw_rectangle(b.pos.x + ox - 2.0, b.pos.y + oy - 8.0, 4.0, 12.0, c);
            } else {
                draw_circle(b.pos.x + ox, b.pos.y + oy, 4.0, RED);
                draw_circle(b.pos.x + ox, b.pos.y + oy, 2.0, WHITE);
            }
        }

        // particles
        for p in &self.particles {
            let a = 1.0 - p.life / p.max_life;
            draw_circle(
                p.pos.x + ox,
                p.pos.y + oy,
                p.size * a + 0.5,
                Color::new(p.color.r, p.color.g, p.color.b, a),
            );
        }

        // floaters (score popups, pickup names)
        for f in &self.floaters {
            let a = 1.0 - f.life / 1.4;
            let m = measure_text(&f.text, None, 20, 1.0);
            draw_text(
                &f.text,
                f.pos.x - m.width / 2.0,
                f.pos.y,
                20.0,
                Color::new(f.color.r, f.color.g, f.color.b, a),
            );
        }

        // player (crab ship: triangle + claws)
        if self.state != State::GameOver {
            let t = get_time() as f32;
            let blink =
                self.player.invincible > 0.0 && (t * 12.0).fract() < 0.5;
            if !blink {
                let p = vec2(self.player.pos.x + ox, self.player.pos.y + oy);
                if self.fx.shield {
                    draw_circle_lines(p.x, p.y, 24.0, 2.5, SKYBLUE);
                }
                if self.fx.magnet > 0.0 {
                    draw_circle_lines(
                        p.x,
                        p.y,
                        20.0 + (t * 4.0).sin() * 3.0,
                        1.5,
                        Color::new(0.3, 0.5, 1.0, 0.5),
                    );
                }
                draw_triangle(p + vec2(0.0, -18.0), p + vec2(-14.0, 12.0), p + vec2(14.0, 12.0), SKYBLUE);
                draw_triangle(p + vec2(0.0, -10.0), p + vec2(-7.0, 6.0), p + vec2(7.0, 6.0), DARKBLUE);
                // claws / wings
                draw_circle(p.x - 16.0, p.y + 6.0, 5.0, ORANGE);
                draw_circle(p.x + 16.0, p.y + 6.0, 5.0, ORANGE);
                // engine flame
                let f = 8.0 + (t * 30.0).sin() * 3.0;
                draw_triangle(
                    p + vec2(-5.0, 12.0),
                    p + vec2(5.0, 12.0),
                    p + vec2(0.0, 12.0 + f),
                    YELLOW,
                );
            }
        }

        // HUD
        draw_text(format!("SCORE {}", self.score).as_str(), 12.0, 28.0, 24.0, WHITE);
        draw_text(
            format!("WAVE {}", self.wave).as_str(),
            w - 130.0,
            28.0,
            24.0,
            LIGHTGRAY,
        );
        draw_text(
            format!("HI {}", self.hi_score.max(self.score)).as_str(),
            w / 2.0 - 40.0,
            28.0,
            24.0,
            GOLD,
        );
        if self.muted {
            draw_text("MUTED", w - 90.0, h - 16.0, 16.0, GRAY);
        }
        // active effect bars under the score
        for (i, (letter, color, rem, total)) in self.fx.active().iter().enumerate() {
            let y = 40.0 + i as f32 * 16.0;
            draw_text(&letter.to_string(), 14.0, y, 14.0, *color);
            draw_rectangle(30.0, y - 10.0, 60.0, 7.0, Color::new(1.0, 1.0, 1.0, 0.15));
            let frac = if *total > 1.0 { rem / total } else { 1.0 };
            draw_rectangle(30.0, y - 10.0, 60.0 * frac.clamp(0.0, 1.0), 7.0, *color);
        }
        // lives as little ships
        for i in 0..self.player.lives {
            draw_triangle(
                vec2(24.0 + i as f32 * 26.0, h - 30.0),
                vec2(14.0 + i as f32 * 26.0, h - 14.0),
                vec2(34.0 + i as f32 * 26.0, h - 14.0),
                SKYBLUE,
            );
        }

        if self.enemies.is_empty()
            && self.powerups.is_empty()
            && self.state == State::Playing
        {
            let msg = format!("WAVE {} INCOMING", self.wave + 1);
            let m = measure_text(&msg, None, 32, 1.0);
            draw_text(&msg, w / 2.0 - m.width / 2.0, h / 2.0, 32.0, YELLOW);
        }

        // bomb / pickup flash
        if self.flash > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                w,
                h,
                Color::new(1.0, 1.0, 1.0, (self.flash * 1.4).min(0.6)),
            );
        }

        if self.state == State::Paused {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.6));
            let t = "PAUSED";
            let m = measure_text(t, None, 48, 1.0);
            draw_text(t, w / 2.0 - m.width / 2.0, h / 2.0 - 60.0, 48.0, WHITE);
            let labels: Vec<String> =
                PAUSE_ITEMS.iter().map(|s| s.to_string()).collect();
            menu::draw_items(w, h, &labels, self.pause_sel);
        }

        if self.state == State::GameOver {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.65));
            let t = "GAME OVER";
            let m = measure_text(t, None, 56, 1.0);
            draw_text(t, w / 2.0 - m.width / 2.0, h / 2.0 - 90.0, 56.0, RED);
            let s = format!(
                "score {}   best {}   wave {}",
                self.score, self.hi_score, self.wave
            );
            let ms = measure_text(&s, None, 24, 1.0);
            draw_text(&s, w / 2.0 - ms.width / 2.0, h / 2.0 - 48.0, 24.0, WHITE);
            let labels: Vec<String> =
                OVER_ITEMS.iter().map(|s| s.to_string()).collect();
            menu::draw_items(w, h, &labels, self.over_sel);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    game.sfx = Sfx::load().await;
    if !game.muted {
        game.sfx.start_music(game.volume);
    }
    loop {
        match game.state {
            State::Menu => game.update_menu(),
            State::Playing => game.update_playing(),
            State::Paused => game.update_paused(),
            State::GameOver => game.update_gameover(),
        }
        // global quit
        if game.quit
            || (is_key_pressed(KeyCode::Q)
                && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)))
        {
            game.save_hi();
            break;
        }
        game.draw();
        next_frame().await;
    }
}
