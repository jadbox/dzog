# Phase 2 Lighting & Radiative Field Plan (`docs/lighting_phas2.md`)

This document outlines the **Phase 2 Lighting Architecture** for the Dzog wave-matter engine (`dzog-core`, `dzog-wgpu`, `asteroids`, and `shmup`), building on the Phase 1 scalar reaction-diffusion luminescence field $L(\mathbf{x}, t)$ and 2.5D topographic elevation/horizon-shadow pipeline.

---

## 1. Plain Explanation & Architectural Goal

In Phase 1, Dzog introduced:
1. A single-channel scalar **exciton luminescence field** $L(\mathbf{x}, t) \in [0, 4]$ (`luminescence.wgsl`) that diffuses outward from solitons, crystal fractures, and explosions.
2. A **2.5D topographic heightfield** $h(\mathbf{x}) \in [0, 1]$ with a single-pass $O(\text{cells})$ directional solar horizon sweep (`MaterialField::recompute_horizon_shadows`).

While Phase 1 provides zero-sprite fire/smoke and directional sun shadows, it has three visual limitations:
- **Monochromatic Field Glow**: Because `Luminescence` only uses the `.r` channel of its `RGBA32Float` GPU texture (`(L, 0, 0, 0)`), cyan plasma bolts, emerald biodome cores, and orange-crimson detonations cannot cast colored light onto nearby ships, canyon walls, or crystal facets simultaneously.
- **No Dynamic Shadow Occlusion for Local Lights**: Diffusing $L(\mathbf{x}, t)$ flows around corners via isotropic Laplacian diffusion ($\nabla^2 L$), but bright explosions and laser beams do not cast sharp radial shadows behind tall geodesic domes or canyon cliffs.
- **Flat Specular/Normal Interaction on Sprites & Terrain**: Ground tiles, water canals, and baked 2.5D ship/building normal maps (`*_height_normal.png`) only respond to static top-left solar lighting rather than real-time $\nabla \mathbf{L}$ directional light vectors from passing projectiles and blasts.

**Why Phase 2 Matters to the Engine**:
Following James Hillman's *situated image* principle (`philosophy.md`), lighting should never be a detached post-process or an $O(N_{\text{lights}} \times N_{\text{pixels}})$ point-light loop. By utilizing the already-allocated `.rgb` channels of the `Luminescence` GPU texture for **chromatic radiative transport** and coupling diffusion to the 2.5D elevation/impedance grid, every projectile, glowing crack, and reactor core naturally illuminates and shadows the world in **$O(1)$ per pixel**.

---

## 2. Core Pillars of Phase 2 Lighting

### Pillar A: Chromatic 3-Band Radiative Field $\mathbf{L}(\mathbf{x}, t) = (L_R, L_G, L_B, S_{\text{soot}})$
The GPU `lum_texture_a` / `lum_texture_b` (`RGBA32Float`) already allocates 16 bytes per cell, with `.gba` currently unused (`0.0`). Phase 2 upgrades `luminescence.wgsl` and `AcousticState` to evolve all four channels:
- **`.r` ($L_R$ — Thermal / Solar / Detonation Band)**: Driven by explosions (`inject_fiery_field_explosion`), thruster plumes, and `ObsidianAnvil` / `BismuthCrystal` fracture heating. Fast peak decay ($\gamma_R = 2.1$) cooling into `.a` soot.
- **`.g` ($L_G$ — Exciton / Piezo / Biodome Band)**: Driven by `PowerCube` annealing condensates, `BiodomeReactor` cores, and stressed `PiezoQuartz` waveguides. Medium decay ($\gamma_G = 1.6$).
- **`.b` ($L_B$ — Cherenkov / Soliton / Shield Band)**: Driven by high-velocity player `SolitonBolt`s, `CollimatedLaserBeam`s, and `AntiphaseShieldPulse` parries. Crisp, short-range diffusion ($D_B = 0.14 c$, $\gamma_B = 2.4$).
- **`.a` ($S_{\text{soot}}$ — Advected Smoke & Soot Density)**: As thermal $L_R$ decays, it feeds a conserved smoke/soot density channel $S_{\text{soot}} \in [0, 1]$ advected by the lattice particle velocity $\mathbf{v} = (v_x, v_y)$:
  $$\frac{\partial S_{\text{soot}}}{\partial t} + \mathbf{v} \cdot \nabla S_{\text{soot}} = D_S \nabla^2 S_{\text{soot}} + \eta_{\text{condense}} L_R - \gamma_S S_{\text{soot}}$$
  This decouples lingering dark charcoal smoke clouds from active light emission, so cooling smoke darkens and occludes underlying terrain without glowing.

### Pillar B: Elevation & Impedance-Coupled Radiative Occlusion (2.5D Shadowed Diffusion)
To prevent light from bleeding straight through solid canyon walls or tall buildings without expensive per-light raymarching:
1. **Topographic Diffusion Barrier in `luminescence.wgsl`**:
   - Pass the 2.5D building/wall elevation $h(\mathbf{x})$ (or high-impedance wall mask) into the luminescence pass.
   - Weight the 4-neighbor Laplacian flux between cell $\mathbf{x}$ and neighbor $\mathbf{x}'$ by an elevation occlusion factor:
     $$T_{\text{light}}(\mathbf{x} \to \mathbf{x}') = \exp\!\left(-\sigma_h \max(0, \; h(\mathbf{x}') - h(\mathbf{x}))\right) \cdot (1 - 0.85 \, \mathbb{I}_{\text{wall}}(\mathbf{x}'))$$
   - Light emitted at low altitude ($z_{\text{bolt}} < h_{\text{dome}}$) is physically blocked by geodesic domes and canyon walls, automatically creating dark shadow umbras on the far side of structures and asteroids.
2. **Dual-Axis Dynamic Horizon Sweep (Optional Fast Point-Shadow Pass)**:
   - For major detonations (`intensity > 2.5`), run a lightweight 4-direction (or 8-octant) horizon elevation sweep from the blast epicenter over a $32 \times 32$ local tile so buildings cast long, dramatic radial flash-shadows across the tarmac during explosions.

### Pillar C: Per-Subpixel 3D Normal & Specular Field Shading (`∇L` Lambertian + Blinn-Phong)
At `960 × 540` sub-cell resolution (`RENDER_SCALE = 4`), every ship, asteroid, building dome, and canal wave already has a 3D surface normal $\mathbf{n} = (n_x, n_y, n_z)$:
- ** Asteroids**: Analytical mineral facet normal from `CohesiveBody::sample_crystal_facet(lx, ly)`.
- **Shmup Ships & Domes**: Baked 4-channel `(nx, ny, nz, elevation_h)` normal maps from `tools/flux-forge` (`*_height_normal.png`) and spherical/octagonal dome profiles (`sample_building_height_and_normal`).
- **Terrain & Hydro-Canals**: Slope normal from $-\nabla h(\mathbf{x})$ and water wave normal from $(-\partial_x p, -\partial_y p, 1)$.

**Phase 2 Sub-Pixel Shading Law**:
1. Compute the local chromatic light vector from the spatial gradient of the luminescence field:
   $$\mathbf{g}_c = -\nabla L_c(\mathbf{x}) \quad \text{for } c \in \{R, G, B\}$$
   and normalized 3D light direction $\hat{\boldsymbol{\ell}}_c = \frac{(-g_{c,x}, \; -g_{c,y}, \; h_{\text{light}} - h(\mathbf{x}))}{\|\cdot\|}$.
2. Evaluate **Lambertian diffuse + mineralogical specular highlight**:
   $$I_c(\mathbf{x}) = L_{c,\text{ambient}}(\mathbf{x}) + k_d \max(0, \; \mathbf{n} \cdot \hat{\boldsymbol{\ell}}_c) \, L_c(\mathbf{x}) + k_s \max(0, \; \mathbf{n} \cdot \mathbf{h}_c)^{\alpha_{\text{spec}}} \, L_c(\mathbf{x})$$
   - **Metallic Hulls & Obsidian (`ObsidianAnvil`)**: High specular exponent ($\alpha_{\text{spec}} = 16, k_s = 0.85$), producing crisp cyan/orange rim reflections when bolts or explosions pass nearby.
   - **Hydro-Canal Water**: Reflects sky projectiles, thruster plumes, and explosion fireballs across animated water ripples ($-\nabla p$).
   - **Internal Crystal Waveguides (`0.15 <= C < 0.68`)**: Sub-surface scattering term proportional to $(1 - C) \mathbf{L}(\mathbf{x})$, making cracked domes and fractured enemy wings glow from within when hit by shockwaves.

---

## 3. Implementation Roadmap

| Step | Target Crate / File | Work Item |
| :--- | :--- | :--- |
| **Step 1: RGB+Soot GPU & CPU Stencil** | `dzog-wgpu/src/shaders/luminescence.wgsl`, `dzog-core/src/solver.rs`, `dzog-core/src/grid.rs` | Expand `luminescence` from scalar `f32` to 4-channel `[f32; 4]` (`L_r, L_g, L_b, soot`) or packed `ChromaticLuminescence` with velocity-advected soot and elevation-gated diffusion $T_{\text{light}}$. Verify CPU/GPU parity in `gpu_cpu_agreement.rs`. |
| **Step 2: Chromatic Ordnance & Fracture Emitters** | `dzog-core/src/excitation.rs`, `dzog-core/src/ordnance.rs`, `shmup/src/combat.rs` | Route player bolts/shields into $L_B$ (cyan-blue), `PowerCube`s/biodomes into $L_G$ (emerald), and explosions/fractures/thrusters into $L_R + S_{\text{soot}}$ (thermal gold-orange $\to$ charcoal smoke). |
| **Step 3: 2.5D Normal-Mapped Sub-Pixel Shading** | `shmup/src/render/{mod,ships,structures}.rs`, `asteroids/src/render.rs` | Sample bilinear $\mathbf{L}$ and $-\nabla \mathbf{L}$ across terrain, water canals, 3D building domes, and ship hulls; apply VGA-quantized specular rim lighting and smoke occlusion. |
| **Step 4: Verification & Visual Toy Scenes** | `shmup/src/main.rs`, `shmup/tests/physics_validation.rs` | Add `--isolate` lighting verification scenes (cyan bolt grazing a dome in darkness, biodome emerald glow on water canal, explosion shadow umbra behind a tower) and assert bounded long-run energy/luminescence stability. |

---

## 4. Performance & Numerical Guardrails

1. **Zero Extra GPU Render Passes**: Keep `luminescence.wgsl` as Pass 3 inside the existing single-submit compute encoder (`GpuSimulation::encode_substeps`). Utilizing `.rgba` of the already-bound `RGBA32Float` luminescence texture adds **zero** additional texture bindings or readback barriers.
2. **Strict Sub-Threshold Gating**: Gate wave-to-light conversion ($|p| > 0.36$) and clamp per-channel $\mathbf{L} \in [0, 4]^3$ with exponential decay $\exp(-\gamma_c \Delta t)$ so background acoustic ripples never ignite runaway glow.
3. **5-Bit VGA Palette Discipline (`shmup`)**: Quantize final lit RGB values through `quantize_vga` (`(v * 31.0).round() / 31.0`) so dynamic colored lighting and specular highlights preserve the crisp 1994 MS-DOS VGA aesthetic (`Raptor: Call of the Shadows`) rather than looking like modern blurry bloom.
