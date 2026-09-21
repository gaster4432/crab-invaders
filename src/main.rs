use macroquad::prelude::*;

const PLAYER_SPEED: f32 = 420.0;
const BULLET_SPEED: f32 = 620.0;
const ENEMY_BULLET_SPEED: f32 = 260.0;
const FIRE_COOLDOWN: f32 = 0.16;

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
    dead: bool,
}

struct Enemy {
    pos: Vec2,
    vel: Vec2,
    kind: EnemyKind,
    hp: i32,
    fire_timer: f32,
    t: f32,
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
    stars: Vec<Star>,
    score: u32,
    hi_score: u32,
    wave: u32,
    spawn_timer: f32,
    shake: f32,
}

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

impl Game {
    fn new() -> Self {
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
            stars,
            score: 0,
            hi_score: 0,
            wave: 1,
            spawn_timer: 0.0,
            shake: 0.0,
        }
    }

    fn reset(&mut self) {
        if self.score > self.hi_score {
            self.hi_score = self.score;
        }
        self.player = Player {
            pos: vec2(screen_width() / 2.0, screen_height() - 80.0),
            cooldown: 0.0,
            lives: 3,
            invincible: 1.0,
        };
        self.bullets.clear();
        self.enemies.clear();
        self.particles.clear();
        self.score = 0;
        self.wave = 1;
        self.spawn_timer = 0.0;
        self.shake = 0.0;
        self.state = State::Playing;
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
                dead: false,
            });
        }
    }

    fn update_menu(&mut self) {
        self.update_stars();
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.reset();
            self.spawn_wave();
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

    fn update_playing(&mut self) {
        let dt = get_frame_time();
        let w = screen_width();
        let h = screen_height();

        self.update_stars();

        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            self.state = State::Paused;
            return;
        }

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
            self.player.cooldown = FIRE_COOLDOWN;
            self.bullets.push(Bullet {
                pos: self.player.pos + vec2(-8.0, -14.0),
                vel: vec2(0.0, -BULLET_SPEED),
                friendly: true,
                dead: false,
            });
            self.bullets.push(Bullet {
                pos: self.player.pos + vec2(8.0, -14.0),
                vel: vec2(0.0, -BULLET_SPEED),
                friendly: true,
                dead: false,
            });
        }

        // --- spawn next wave when cleared ---
        if self.enemies.is_empty() {
            self.spawn_timer += dt;
            if self.spawn_timer > 1.2 {
                self.wave += 1;
                self.spawn_timer = 0.0;
                self.spawn_wave();
                // small heal every 5 waves
                if self.wave % 5 == 0 && self.player.lives < 5 {
                    self.player.lives += 1;
                }
            }
        } else {
            self.spawn_timer = 0.0;
        }

        // --- enemies ---
        let player_pos = self.player.pos;
        let mut new_enemy_bullets: Vec<Bullet> = Vec::new();
        for e in &mut self.enemies {
            e.t += dt;
            match e.kind {
                EnemyKind::Diver => {
                    // drift toward player x
                    let dx = player_pos.x - e.pos.x;
                    e.pos.x += dx.clamp(-1.0, 1.0) * 60.0 * dt;
                    e.pos += e.vel * dt;
                }
                EnemyKind::Weaver => {
                    e.pos += e.vel * dt;
                    e.pos.x += (e.t * 3.0).sin() * 120.0 * dt;
                    if e.pos.x < 16.0 || e.pos.x > w - 16.0 {
                        e.vel.x = -e.vel.x;
                        e.pos.x = e.pos.x.clamp(16.0, w - 16.0);
                    }
                }
                EnemyKind::Tank => {
                    e.pos += e.vel * dt;
                }
            }
            // wrap / floor: divers that pass bottom re-enter top
            if e.pos.y > h + 30.0 {
                e.pos.y = -30.0;
                e.pos.x = rand::gen_range(20.0, w - 20.0);
            }
            // shooting
            if e.kind != EnemyKind::Diver {
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
        for bi in 0..self.bullets.len() {
            if !self.bullets[bi].friendly || self.bullets[bi].dead {
                continue;
            }
            let bpos = self.bullets[bi].pos;
            for ei in 0..self.enemies.len() {
                if self.enemies[ei].dead {
                    continue;
                }
                let r = if self.enemies[ei].kind == EnemyKind::Tank {
                    18.0
                } else {
                    13.0
                };
                if bpos.distance(self.enemies[ei].pos) < r + 4.0 {
                    self.bullets[bi].dead = true;
                    self.enemies[ei].hp -= 1;
                    boom_events.push((bpos, YELLOW, 4));
                    if self.enemies[ei].hp <= 0 {
                        self.enemies[ei].dead = true;
                        let (pts, col) = match self.enemies[ei].kind {
                            EnemyKind::Diver => (50, RED),
                            EnemyKind::Weaver => (100, ORANGE),
                            EnemyKind::Tank => (250, PURPLE),
                        };
                        score_add += pts;
                        boom_events.push((self.enemies[ei].pos, col, 22));
                        shake_add += 6.0;
                    }
                    break;
                }
            }
        }
        self.score += score_add;
        self.shake = (self.shake + shake_add).min(12.0);
        for (pos, col, n) in boom_events {
            spawn_explosion(&mut self.particles, pos, col, n);
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
                self.player.lives -= 1;
                self.player.invincible = 2.0;
                let ppos = self.player.pos;
                spawn_explosion(&mut self.particles, ppos, SKYBLUE, 30);
                self.shake = 14.0;
                if self.player.lives <= 0 {
                    if self.score > self.hi_score {
                        self.hi_score = self.score;
                    }
                    self.state = State::GameOver;
                    return;
                }
            }
        }

        self.bullets.retain(|b| !b.dead);
        self.enemies.retain(|e| !e.dead);

        // --- particles ---
        for p in &mut self.particles {
            p.life += dt;
            p.pos += p.vel * dt;
            p.vel *= 1.0 - 2.2 * dt;
        }
        self.particles.retain(|p| p.life < p.max_life);

        self.shake = (self.shake - dt * 30.0).max(0.0);
    }

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
            let title = "CRAB INVADERS";
            let ts = measure_text(title, None, 64, 1.0);
            draw_text(
                title,
                w / 2.0 - ts.width / 2.0,
                h / 2.0 - 60.0,
                64.0,
                ORANGE,
            );
            let sub = "a tiny arcade shooter";
            let ss = measure_text(sub, None, 24, 1.0);
            draw_text(
                sub,
                w / 2.0 - ss.width / 2.0,
                h / 2.0 - 20.0,
                24.0,
                LIGHTGRAY,
            );
            let help = [
                "ARROWS / WASD - move",
                "SPACE - shoot",
                "P - pause    ENTER - start",
            ];
            for (i, line) in help.iter().enumerate() {
                let m = measure_text(line, None, 20, 1.0);
                draw_text(
                    line,
                    w / 2.0 - m.width / 2.0,
                    h / 2.0 + 30.0 + i as f32 * 28.0,
                    20.0,
                    GRAY,
                );
            }
            if self.hi_score > 0 {
                let hs = format!("HI-SCORE  {}", self.hi_score);
                let m = measure_text(&hs, None, 20, 1.0);
                draw_text(hs.as_str(), w / 2.0 - m.width / 2.0, h / 2.0 + 140.0, 20.0, GOLD);
            }
            return;
        }

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

        // bullets
        for b in &self.bullets {
            if b.friendly {
                draw_rectangle(b.pos.x + ox - 2.0, b.pos.y + oy - 8.0, 4.0, 12.0, YELLOW);
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

        // player (crab ship: triangle + claws)
        if self.state != State::GameOver {
            let t = get_time() as f32;
            let blink =
                self.player.invincible > 0.0 && (t * 12.0).fract() < 0.5;
            if !blink {
                let p = vec2(self.player.pos.x + ox, self.player.pos.y + oy);
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
        // lives as little ships
        for i in 0..self.player.lives {
            draw_triangle(
                vec2(24.0 + i as f32 * 26.0, h - 30.0),
                vec2(14.0 + i as f32 * 26.0, h - 14.0),
                vec2(34.0 + i as f32 * 26.0, h - 14.0),
                SKYBLUE,
            );
        }

        if self.enemies.is_empty() && self.state == State::Playing {
            let msg = format!("WAVE {} INCOMING", self.wave + 1);
            let m = measure_text(&msg, None, 32, 1.0);
            draw_text(&msg, w / 2.0 - m.width / 2.0, h / 2.0, 32.0, YELLOW);
        }

        if self.state == State::Paused {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.6));
            let t = "PAUSED - press P to resume";
            let m = measure_text(t, None, 28, 1.0);
            draw_text(t, w / 2.0 - m.width / 2.0, h / 2.0, 28.0, WHITE);
        }

        if self.state == State::GameOver {
            draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, 0.65));
            let t = "GAME OVER";
            let m = measure_text(t, None, 56, 1.0);
            draw_text(t, w / 2.0 - m.width / 2.0, h / 2.0 - 20.0, 56.0, RED);
            let s = format!("score {}   best {}", self.score, self.hi_score);
            let ms = measure_text(&s, None, 24, 1.0);
            draw_text(&s, w / 2.0 - ms.width / 2.0, h / 2.0 + 24.0, 24.0, WHITE);
            let r = "ENTER to retry   ESC to quit";
            let mr = measure_text(r, None, 20, 1.0);
            draw_text(r, w / 2.0 - mr.width / 2.0, h / 2.0 + 60.0, 20.0, GRAY);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    loop {
        match game.state {
            State::Menu => game.update_menu(),
            State::Playing => game.update_playing(),
            State::Paused => {
                game.update_stars();
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Enter) {
                    game.state = State::Playing;
                }
                if is_key_pressed(KeyCode::Q) {
                    game.state = State::Menu;
                }
            }
            State::GameOver => {
                game.update_stars();
                // let particles finish
                let dt = get_frame_time();
                for p in &mut game.particles {
                    p.life += dt;
                    p.pos += p.vel * dt;
                }
                game.particles.retain(|p| p.life < p.max_life);
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    game.reset();
                    game.spawn_wave();
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.state = State::Menu;
                }
            }
        }
        // global quit
        if is_key_pressed(KeyCode::Q) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            break;
        }
        game.draw();
        next_frame().await;
    }
}
