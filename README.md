Knockoff Tetris
======

## About

This is clone of Tetris (with a much simplified rotation system), written in Rust using the Bevy engine. The purpose of this program is twofold:

- Learn the Rust programming language
- Familiarize myself with the Bevy game engine
- Play around with different code organization

Features implemented:
- Soft and hard dropping
- Lock delay
- Kicks as defined by the SRS ([Super Rotation System](https://tetris.fandom.com/wiki/SRS))
- Generic kick system
- Ghost blocks

Features not yet implemented:
- Random-from-a-bag picking of the next Tetromino
- Any scorekeeping
- Game-over detection
- Animated block movement
- 3D scene of blocks

## Controls

- `A` - rotate left
- `D` - rotate right
- `< LEFT` / `RIGHT >` - move block
- `DOWN v` - soft-drop block
- `UP ^` - hard-drop block
- `SPACE` - (for debugging) - pause / unpause block dropping

## Building

```bash
cargo check
cargo run
```

## Screenshots
![](./screens/1.png)