# Smol World

A tiny cube-planet garden game built with [Bevy](https://bevyengine.org/).

## Features

- **Cube Planet**: Walk around all 6 faces of a small cube world
- **Isometric Camera**: Orthographic projection that orbits around the cube (right-click drag or Q/E keys)
- **Garden**: A colorful flower garden on the planet surface
- **House**: A small house with walls, roof, and chimney
- **Occlusion Transparency**: Objects between the camera and player become transparent so the player is always visible, even when the camera rotates

## Controls

| Input | Action |
|-------|--------|
| WASD / Arrow Keys | Move player |
| Right Mouse Drag | Orbit camera |
| Q / E | Rotate camera left/right |
| Scroll Wheel | Zoom in/out |

## Building

### Native (desktop)

```bash
cargo run --release
```

### WebAssembly

```bash
# Install wasm-bindgen-cli if not already installed
cargo install wasm-bindgen-cli

# Build for WASM
cargo build --release --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen --out-dir web --target web target/wasm32-unknown-unknown/release/smol_world.wasm

# Serve the web/ directory with any static file server
# e.g. python3 -m http.server -d web
```

## Architecture

The game uses a simple ECS architecture with Bevy:

- **Orbit Camera System**: Maintains a fixed-distance orthographic camera that orbits the cube planet
- **Player Movement**: Camera-relative WASD movement that keeps the player clamped to cube faces
- **Occlusion System**: Each frame, casts a ray from camera to player and fades any `Occludable` entities that intersect, creating a see-through effect for the house
