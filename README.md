# Crab Invaders

A fast 2D arcade space shooter written in Rust + [macroquad](https://github.com/not-fl3/macroquad).
No assets, no config — just shoot.

![license](https://img.shields.io/badge/license-MIT-blue)

## Play

| Key | Action |
|-----|--------|
| Arrows / WASD | Move |
| Space / J | Shoot |
| P / Esc | Pause |
| M | Mute |
| Enter / Click | Menu select |
| Ctrl+Q | Quit |

3 enemy types, endless waves, hi-score (saved to disk),
particles, screen shake. +1 life every 5 waves (max 5).

Menus: START / HOW TO PLAY / SETTINGS (mute + volume) / QUIT,
pause menu, and game-over screen — all keyboard and mouse driven.

## Power-ups

Kills drop lettered pickups (tanks drop more often):

| Letter | Effect |
|--------|--------|
| S | Spread shot (12s) |
| R | Rapid fire (10s) |
| H | Shield — absorbs one hit |
| B | Bomb — wipes the screen |
| + | +1 life (rare) |
| M | Magnet — pulls pickups, zaps nearby bullets (12s) |
| F | Freeze — slows + disarms enemies (8s) |
| P | Piercing bullets (12s) |
| 2 | Double score (15s) |

## Audio

All sound is synthesized at startup — no audio files.
SFX for shooting, explosions, pickups, UI, plus a looping
chiptune track. `M` mutes; volume in SETTINGS.
Hi-score, mute, and volume persist in
`~/.config/crab-invaders/save`.

## Run from source

```bash
cargo run --release
```

Requires only Rust stable (1.70+). No system gamedev libs beyond
normal X11/GL/ALSA runtime libs that Arch already has.

## Install on Arch (AUR)

```bash
# after pushing to the AUR:
yay -S crab-invaders
# or manual:
git clone https://aur.archlinux.org/crab-invaders.git
cd crab-invaders
makepkg -si
```

Binary installs to `/usr/bin/crab-invaders` plus a `.desktop` entry.

## Packaging notes (for maintainer)

1. Push this repo to GitHub, tag `v0.1.0`:
   ```bash
   git init
   git add .
   git commit -m "Crab Invaders 0.1.0"
   git remote add origin https://github.com/gaster4432/crab-invaders
   git tag v0.1.0
   git push -u origin master --tags
   ```
2. Update `PKGBUILD`:
   - `Maintainer:` line, `url=`, `source=`
   - run `updpkgsums && makepkg --printsrcinfo > .SRCINFO`
3. Test: `makepkg -si` then `namcap PKGBUILD` and `namcap crab-invaders-*.pkg.tar.zst`
4. Publish:
   ```bash
   git clone ssh://aur@aur.archlinux.org/crab-invaders.git aur-pkg
   cp PKGBUILD .SRCINFO crab-invaders.desktop aur-pkg/
   cd aur-pkg && makepkg --printsrcinfo > .SRCINFO
   git add PKGBUILD .SRCINFO crab-invaders.desktop
   git commit -m "0.1.0-1" && git push
   ```

See [Arch Rust packaging guidelines](https://wiki.archlinux.org/title/Rust_package_guidelines).

## Layout

```
Cargo.toml              # macroquad = "0.4", release LTO + strip
src/main.rs             # whole game (~600 lines, zero assets)
PKGBUILD                # AUR source build
crab-invaders.desktop   # desktop entry
LICENSE                 # MIT
```

## License

MIT — see `LICENSE`.
