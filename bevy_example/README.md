Bevy prototype for pet-rs

Run the example with:

```bash
cd bevy_example
cargo run --release
```

Behavior:
- Displays a generated circular sprite (pet) in a transparent window.
- Left-drag the pet to move it inside the window.
- Right-click to open a small UI menu (Pause/Resume, Quit). Menu is a simple Bevy UI prototype.

Notes:
- This is a minimal prototype to demonstrate moving to Bevy. For native window dragging or click-through shaped windows, additional platform-specific code is required.
