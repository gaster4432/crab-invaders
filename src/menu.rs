//! Menu stack: Main / Help / Settings, plus shared button geometry.
//! Pause and Game-Over selections live in `Game` (they need game context).

use crate::powerups::PowerKind;
use macroquad::prelude::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MenuScreen {
    Main,
    Help,
    Settings,
}

pub struct MenuState {
    pub screen: MenuScreen,
    pub selected: usize,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            screen: MenuScreen::Main,
            selected: 0,
        }
    }

    pub fn items(&self, muted: bool, volume: f32) -> Vec<String> {
        match self.screen {
            MenuScreen::Main => vec![
                "START".to_string(),
                "HOW TO PLAY".to_string(),
                "SETTINGS".to_string(),
                "QUIT".to_string(),
            ],
            MenuScreen::Help => vec!["BACK".to_string()],
            MenuScreen::Settings => vec![
                format!("MUTE: {}", if muted { "ON" } else { "OFF" }),
                format!("VOLUME: {}%", (volume * 100.0).round() as i32),
                "BACK".to_string(),
            ],
        }
    }

    pub fn up(&mut self, muted: bool, volume: f32) {
        let n = self.items(muted, volume).len();
        self.selected = (self.selected + n - 1) % n;
    }

    pub fn down(&mut self, muted: bool, volume: f32) {
        let n = self.items(muted, volume).len();
        self.selected = (self.selected + 1) % n;
    }

    pub fn goto(&mut self, screen: MenuScreen) {
        self.screen = screen;
        self.selected = 0;
    }
}

// ---- layout (pure geometry, unit-tested) ----

pub const BUTTON_W: f32 = 340.0;
pub const BUTTON_H: f32 = 38.0;
pub const BUTTON_GAP: f32 = 46.0;

pub fn items_top_y(screen_h: f32) -> f32 {
    screen_h / 2.0 + 10.0
}

pub fn item_rect(index: usize, screen_w: f32, screen_h: f32) -> Rect {
    Rect::new(
        screen_w / 2.0 - BUTTON_W / 2.0,
        items_top_y(screen_h) + index as f32 * BUTTON_GAP,
        BUTTON_W,
        BUTTON_H,
    )
}

pub fn clicked_on(rect: &Rect, mx: f32, my: f32) -> bool {
    rect.contains(vec2(mx, my))
}

pub fn draw_title(w: f32, h: f32, hi: u32) {
    let title = "SHELL SHOOTER";
    let ts = measure_text(title, None, 64, 1.0);
    draw_text(title, w / 2.0 - ts.width / 2.0, h / 2.0 - 90.0, 64.0, ORANGE);
    let sub = "a tiny arcade shooter";
    let ss = measure_text(sub, None, 24, 1.0);
    draw_text(sub, w / 2.0 - ss.width / 2.0, h / 2.0 - 52.0, 24.0, LIGHTGRAY);
    if hi > 0 {
        let hs = format!("HI-SCORE  {hi}");
        let m = measure_text(&hs, None, 22, 1.0);
        draw_text(&hs, w / 2.0 - m.width / 2.0, h / 2.0 - 18.0, 22.0, GOLD);
    }
}

pub fn draw_items(w: f32, h: f32, items: &[String], selected: usize) {
    for (i, label) in items.iter().enumerate() {
        let r = item_rect(i, w, h);
        let sel = i == selected;
        draw_rectangle(
            r.x,
            r.y,
            r.w,
            r.h,
            if sel {
                Color::new(0.95, 0.55, 0.15, 0.25)
            } else {
                Color::new(1.0, 1.0, 1.0, 0.06)
            },
        );
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if sel { ORANGE } else { DARKGRAY });
        let m = measure_text(label, None, 22, 1.0);
        draw_text(
            label,
            w / 2.0 - m.width / 2.0,
            r.y + 27.0,
            22.0,
            if sel { WHITE } else { LIGHTGRAY },
        );
    }
    let hint = "UP/DOWN + ENTER   -   CLICK WORKS TOO";
    let m = measure_text(hint, None, 16, 1.0);
    draw_text(
        hint,
        w / 2.0 - m.width / 2.0,
        h - 24.0,
        16.0,
        Color::new(1.0, 1.0, 1.0, 0.35),
    );
}

pub fn draw_help(w: f32, _h: f32) {
    let y0 = 96.0;
    draw_text("HOW TO PLAY", w / 2.0 - 110.0, y0, 36.0, ORANGE);
    let lines = [
        "ARROWS / WASD - move     SPACE / J - shoot",
        "P - pause     M - mute     CTRL+Q - quit",
        "",
        "ENEMIES:  red divers chase - orange weavers shoot",
        "purple tanks take 3 hits - +1 life every 5 waves",
        "",
        "POWER-UPS (grab the falling letters):",
    ];
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, 60.0, y0 + 40.0 + i as f32 * 24.0, 18.0, LIGHTGRAY);
    }
    // power-up legend in two columns
    let kinds = PowerKind::ALL;
    for (i, k) in kinds.iter().enumerate() {
        let col = (i % 2) as f32;
        let row = (i / 2) as f32;
        let x = 60.0 + col * 340.0;
        let y = y0 + 40.0 + 7.0 * 24.0 + row * 26.0;
        draw_circle(x + 8.0, y - 6.0, 10.0, k.color());
        let letter = k.letter().to_string();
        let m = measure_text(&letter, None, 16, 1.0);
        draw_text(&letter, x + 8.0 - m.width / 2.0, y, 16.0, BLACK);
        draw_text(k.name(), x + 26.0, y, 18.0, WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_wraps_around() {
        let mut m = MenuState::new();
        assert_eq!(m.items(false, 0.8).len(), 4);
        m.up(false, 0.8);
        assert_eq!(m.selected, 3);
        m.down(false, 0.8);
        assert_eq!(m.selected, 0);
    }

    #[test]
    fn settings_labels_reflect_state() {
        let m = MenuState {
            screen: MenuScreen::Settings,
            selected: 0,
        };
        let items = m.items(true, 0.5);
        assert_eq!(items[0], "MUTE: ON");
        assert_eq!(items[1], "VOLUME: 50%");
        assert_eq!(items[2], "BACK");
    }

    #[test]
    fn button_rects_dont_overlap_and_center() {
        let w = 800.0;
        let h = 600.0;
        let a = item_rect(0, w, h);
        let b = item_rect(1, w, h);
        assert!(!a.overlaps(&b));
        assert!((a.x + a.w / 2.0 - w / 2.0).abs() < 0.01);
        assert!(clicked_on(&a, w / 2.0, a.y + 5.0));
        assert!(!clicked_on(&a, 10.0, 10.0));
    }
}
