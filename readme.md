# Dzog Simulator
<small>by [Jonathan Dunlap](https://www.linkedin.com/in/jonathandunlap/)</small>

**Note: This is highly experimental work for a proof-of-concept simulator inspired by quantum field mechanics.**

**Dzog Simulator** is a GPU-accelerated 2D wave-matter field-physics engine in Rust.

Conventional game engines treat space as an empty coordinate system populated by isolated rigid bodies (`struct Asteroid { hp: 2 }`) that ignore one another until a bounding-box check triggers an imperative callback. **Dzog Simulator** replaces object-centric execution with a continuous physical medium governed by nonlinear wave mechanics, solid-state crystal cohesion, and pilot-wave guidance.

In Dzog Simulator, a scene is an **executable physical situation**: a lattice where every cell carries pressure, velocity, material impedance, and structural cohesion. Ships, destructible asteroids, projectiles, and shields are persistent patterns of matter and wave excitation that radiate wakes, fracture along continuous stress planes, and interfere directly across the medium.

## Interactive Demos

### 1. Crystalline Asteroids & Alien Ecology
> **Live Web Demo:** [**Play in Browser (`jadbox.github.io/dzog/asteroids/`)**](https://jadbox.github.io/dzog/asteroids/)  
> Toroidal & gravity-well wave mechanics, topological crystal cleaving, antiphase shields, and Bohm-Hiley pilot-wave navigation.

https://github.com/user-attachments/assets/63759ad7-6401-47e1-8674-87e3e15a4092

### 2. Crystalline Corridor (Vertical Wave-Stream Shmup)
> **Live Web Demo:** [**Play in Browser (`jadbox.github.io/dzog/shmup/`)**](https://jadbox.github.io/dzog/shmup/)  
> A scrolling metamaterial canyon featuring a **Tri-Lobe Crystalline Ship** (where wing lobes physically sever and re-crystallize instead of using integer hitpoints), **Chladni Pilot-Wave Swarms** surfing standing-wave pockets, and contested **Annealing Power Cubes**.

https://github.com/user-attachments/assets/d11876b8-0870-47fe-8872-23fe1d393944

---

## Core Concepts: The Field as Computation

Just as a raster display uses a 2D array of *pixels* for color, Dzog uses a 2D array of coupled **field cells** for physics and material identity. Because the medium mediates all causality, gameplay mechanics emerge from local field laws rather than $O(N^2)$ object-pair scripts:

1. **Active Space & Material Laws**: Every cell stores wave state alongside density, stiffness, damping, and cohesion. Reflection, refraction, recoil, and acoustic pitch emerge naturally as waves cross material boundaries.
2. **Solitary Wave Projectiles (Solitons)**: Standard waves disperse like pond ripples. Using Louis de Broglie's *Nonlinear Wave Mechanics*, Dzog adds a saturating self-focusing law so plasma bolts and lasers hold together as compact, particle-like wave packets across the screen.
3. **Pilot-Wave Navigation (Active Information)**: Drawing on David Bohm and Basil Hiley's *Quantum Potential ($Q$)*, the geometry (spatial curvature) of the wavefield guides actors. Alien squadrons and Chladni swarms naturally surf standing-wave valleys without pathfinding graphs.
4. **Topological Fracture & Annealing**: Solid structures are regions of high cell cohesion rather than integer hitpoint counters (`hp: 3`). Overpressure carves internal crack waveguides until bodies cleave topologically into tumbling fragments; capturing piezoelectric condensates re-anneals severed crystal wings cell by cell.
5. **Unified Sight & Sound**: Following James Hillman's *situated image*, visuals, 3D crystal lighting, and real-time Doppler audio are direct projections of the same evolving field state—requiring zero `.wav` files or detached particle scripts.

For the full mathematical handbook, intellectual lineage, and GPU architecture, read the [**Field-First Engine Whitepaper (`docs/field_first_whitepaper.md`)**](./docs/field_first_whitepaper.md).

---

## Quick Start

```bash
# Run the full verification suite (analytical, GPU/CPU agreement, and physics tests)
cargo test --workspace

# Launch the Crystalline Corridor Vertical Wave-Stream Shmup
cargo run -p shmup
cargo run -p shmup -- --autopilot

# Launch the Crystalline Asteroids & Alien Ecology demo
cargo run -p asteroids
cargo run -p asteroids -- --autopilot

# Record a 30-second 60 FPS MP4 demo with 48 kHz stereo field audio
cargo run --release -p asteroids -- --autopilot --record asteroids_demo.mp4 --duration 30

# Build web demos locally (writes dist/web)
tools/build_web.sh
```

---

## Workspace Crates

| Crate | Purpose |
| --- | --- |
| [`crates/dzog-core`](./crates/dzog-core) | Pure-Rust reference field solver, nonlinear soliton & pilot-wave physics, topological crystal fracture/annealing, and modal audio synthesis. |
| [`crates/dzog-wgpu`](./crates/dzog-wgpu) | Hardware WGPU compute pipeline (`WGSL` shaders) with zero-readback GPU body force/torque reduction. |
| [`crates/dzog-bevy`](./crates/dzog-bevy) | Bevy engine integration: field canvas rendering, diagnostic view modes, GPU compute scheduling, and native `bevy_audio` streaming. |
| [`crates/shmup`](./crates/shmup) | **Crystalline Corridor**: Vertical wave-stream shmup with Tri-Lobe crystal ships, Chladni swarm carriers, and annealing power cubes. |
| [`asteroids`](./asteroids) | **Crystalline Asteroids**: Toroidal arena with multi-mineral crystal cleavage, antiphase shields, and Bohmian alien craft. |

---

## Documentation

- [**Field-First Engine Whitepaper (`docs/field_first_whitepaper.md`)**](./docs/field_first_whitepaper.md): Comprehensive engine handbook—governing equations, de Broglie / Bohm / Hiley theoretical foundations, Yee-staggered numerics, and WGPU compute layout.
- [**Philosophy (`philosophy.md`)**](./philosophy.md): Relation-first computing, critique of symbolic isolation, and the `Encounter -> Perturb -> Compare -> Commit -> Compose` loop.
- [**Audio Overview (`docs/overview_audio.md`)**](./docs/overview_audio.md): Material-tuned modal synthesis, binaural cockpit probes, and `bevy_audio` integration.

---

## License

Permissive **MIT Attribution License** — see [`LICENSE`](./LICENSE). Forks and redistributions must preserve original author credit (`J. Dunlap / jadbox`).
