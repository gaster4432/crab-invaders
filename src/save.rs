//! Hi-score + settings persistence.
//! Stored as a tiny key=value text file so there are no extra dependencies:
//! `$HOME/.config/shell-shooter/save`

use std::io;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Save {
    pub hi: u32,
    pub muted: bool,
    pub volume: f32, // 0.0..=1.0
}

impl Default for Save {
    fn default() -> Self {
        Self {
            hi: 0,
            muted: false,
            volume: 0.8,
        }
    }
}

impl Save {
    pub fn save_path() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config/shell-shooter/save"))
    }

    pub fn load() -> Self {
        Self::save_path()
            .and_then(|p| Self::load_from(&p).ok())
            .unwrap_or_default()
    }

    pub fn load_from(path: &std::path::Path) -> io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::parse(&text))
    }

    pub fn parse(text: &str) -> Self {
        let mut s = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match k.trim() {
                "hi" => {
                    if let Ok(v) = v.trim().parse() {
                        s.hi = v;
                    }
                }
                "muted" => s.muted = v.trim() == "true",
                "volume" => {
                    if let Ok(v) = v.trim().parse::<f32>() {
                        s.volume = v.clamp(0.0, 1.0);
                    }
                }
                _ => {}
            }
        }
        s
    }

    pub fn render(&self) -> String {
        format!(
            "# shell-shooter save (hi-score + settings)\nhi={}\nmuted={}\nvolume={:.2}\n",
            self.hi, self.muted, self.volume
        )
    }

    pub fn save(&self) {
        if let Some(p) = Self::save_path() {
            let _ = self.save_to(&p);
        }
    }

    pub fn save_to(&self, path: &std::path::Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let s = Save {
            hi: 12345,
            muted: true,
            volume: 0.5,
        };
        assert_eq!(Save::parse(&s.render()), s);
    }

    #[test]
    fn bad_input_falls_back_to_defaults() {
        let s = Save::parse("garbage\nvolume=abc\nhi=-5\nmuted=maybe\n");
        assert_eq!(s, Save::default());
        // comments and blanks are ignored
        let s = Save::parse("# hi\n\nhi=777\n");
        assert_eq!(s.hi, 777);
    }

    #[test]
    fn volume_is_clamped() {
        assert_eq!(Save::parse("volume=9").volume, 1.0);
        assert_eq!(Save::parse("volume=-2").volume, 0.0);
    }

    #[test]
    fn writes_and_reads_a_real_file() {
        let dir = std::env::temp_dir().join("shell-shooter-test-save");
        let path = dir.join("save");
        let s = Save {
            hi: 999,
            muted: false,
            volume: 0.3,
        };
        s.save_to(&path).unwrap();
        assert_eq!(Save::load_from(&path).unwrap(), s);
        std::fs::remove_dir_all(&dir).ok();
    }
}
