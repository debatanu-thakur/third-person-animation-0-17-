# Third Person Animation 0.17

Experimental third-person character animation system for Bevy 0.17, featuring:
- Unity-style animation target matching
- Custom FABRIK IK solver for limb placement
- Overgrowth-inspired procedural animation (experimental)

## Features

### Animation Systems

- **Target Matching**: Unity-style animation target matching for foot and hand placement
- **IK System**: Custom FABRIK IK solver for precise limb control
- **Procedural Animation** (Experimental): Overgrowth-style 13-keyframe pose blending

### Cargo Features

- `dev` - Development features (dynamic linking, dev tools)
- `dev_native` - Native development (includes asset hot-reloading)
- `extract_poses` - Enable pose extraction tool for procedural animation

## Quick Start

```bash
# Run with default features (dev_native)
cargo run

# Extract animation poses (for procedural animation)
cargo run --features extract_poses

# Run with specific features
cargo run --features "dev_native,extract_poses"
```

## Documentation

- [IK Integration Summary](IK_INTEGRATION_SUMMARY.md)
- [Procedural Animation Guide](PROCEDURAL_ANIMATION.md)

## Project Structure

```
src/
├── game/              # Game logic and systems
│   ├── animations/    # Traditional animation system
│   ├── target_matching/ # Unity-style target matching
│   ├── foot_placement.rs
│   └── hand_placement.rs
├── ik/                # Custom FABRIK IK solver
├── procedural_animation/ # Overgrowth-style pose blending
└── main.rs
```

## References

- [Wolfire GDC 2014 - Procedural Animation](https://www.wolfire.com/blog/2014/05/GDC-2014-Procedural-Animation-Video/)
- [Bevy 0.17 Documentation](https://docs.rs/bevy/0.17)
