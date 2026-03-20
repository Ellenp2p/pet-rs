Bevy port of `src/main.rs`

Run:

```bash
cd bevy_port
cargo run --release
```

Notes:
- Loads `assets/spritesheet.png` if present (horizontal or vertical strip), otherwise generates procedural frames.
- Animates frames, supports per-pixel hit testing to drag the pet inside the window.
- Clicking transparent pixels initiates native window drag.
- Right-click opens a simple UI menu with Pause/Resume and Quit.
