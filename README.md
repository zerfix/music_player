# Music Player

A music player TUI.

Attempting to be a more intuitive alternative to cmus.

## Quick start

### Build & run

1. Install rust & nix.
2. Clone repo
3. Run `nix-shell`
4. Run `cargo build --release`
5. Run `./target/release/music_player`
6. Configure config file with path to your music folder as instructed

### Controls

- `w` `a` `s` `d` navigation
- `enter` play track selection
- `space` add track selection to playlist
- `tab` switch filter or selection
- `x` previous track
- `c` play/pause
- `v` stop
- `b` next track
- `q` quit

Full list of controls [here](./src/tasks/listener_input.rs).
