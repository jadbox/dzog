# Field-First Computing: The Dzog Engine Handbook

<small>by [Jonathan Dunlap](https://www.linkedin.com/in/jonathandunlap/)</small>

**Abstract**: Conventional simulation and game engines model virtual worlds as collections of isolated data structures sitting in empty coordinate space. Objects remain inert until an explicit collision check or event bus schedules an imperative callback, requiring brittle $O(N^2)$ state scripts to coordinate cross-entity behavior. **Dzog** implements an alternative paradigm: *relation-first computing* over a continuous physical field. Building on **Louis de Broglie's** *Nonlinear Wave Mechanics* (the Theory of the Double Solution), **David Bohm and Basil Hiley's** process-first formulation of quantum mechanics (*active information* and the Quantum Potential $Q$), and **James Hillman's** concept of the *situated image*, Dzog treats the spatial medium itself as the primary computational substrate. Every actor—whether a projectile, a destructible crystal asteroid, a shield, or an autonomous swarm—is a persistent pattern of wave excitation and material commitment within a shared Hamiltonian lattice. Because the medium mediates all causality, collision response, structural fracture, AI navigation, dynamic lighting, and real-time audio synthesis emerge directly from local physical laws rather than special-case scripts.

---

## 1. The Architectural Problem & Intellectual Lineage

### 1.1 Generalized Overview: Beyond Object Isolation
To a software engineer, the standard architecture of an interactive simulation is second nature: a scene is a list of structs (e.g., `struct Asteroid { pos: Vec2, hp: 3 }`) updated in a loop. Space is merely a coordinate system—a void with no state, memory, or tension.

While efficient for sparse rigid bodies, this object-centric model creates three systemic bottlenecks when building rich, emergent worlds:
1. **Dead Space**: Nothing exists between objects. A near-miss has zero physical consequence unless a developer manually codes a proximity trigger.
2. **Combinatorial Scripting ($O(N^2)$ Complexity)**: Every new interaction—a laser heating a shield, a shockwave cracking a weakened wall, or a drone weaving around debris—requires custom glue code linking specific object pairs.
3. **Binary Destruction**: Damage is reduced to an integer counter (`hp -= 1`) swapped for a pre-baked explosion animation, disconnecting visual appearance from structural mechanics.

### 1.2 Why It Matters to the Engine: The Field-First Inversion
What if the grid of space itself performed the computation?

Just as a raster display uses a 2D array of *pixels* to represent color, Dzog uses a 2D array of coupled **field cells** to represent physical state and material identity. In this architecture:
- **Space is an active medium**: Every cell stores pressure ($p$), velocity ($\mathbf{v}$), density ($\rho$), stiffness ($K$), damping ($\gamma$), and structural cohesion ($C$).
- **Objects are patterns in the field**: A solid ship or asteroid is a region of cells with high cohesion and material density; a plasma bolt is a concentrated packet of wave energy traveling through those cells.
- **Causality is local and automatic**: Instead of checking whether `Projectile A` intersects `Object B`, every cell updates itself from its immediate neighbors. Reflection, recoil, fracture, and sound emerge naturally as waves cross material boundaries.

### 1.3 Whose Work We Build On
To turn a continuous wave grid into a viable engine for interactive games and simulations, Dzog synthesizes insights from five key thinkers across physics, numerical analysis, and philosophy:

| Thinker / Pioneers | Core Concept | What It Solves in the Dzog Engine |
| :--- | :--- | :--- |
| **Louis de Broglie** *(1927, 1956)* | **Nonlinear Wave Mechanics & The Double Solution ($u = u_0 + v$)** | Linear waves spread out and fade like pond ripples. De Broglie proposed that particles are localized, nonlinear wave concentrations (**solitons**, $u_0$) embedded in a surrounding phase wave ($v$) and locked by *Phase Harmony*. Dzog uses this to make projectiles travel as crisp, self-focusing wave packets. |
| **David Bohm & Basil Hiley** *(1952, 1993)* | **Active Information & The Quantum Potential ($Q = -\nabla^2 R / R$)** | How does a wave steer an actor without brute-force shoving? Bohm and Hiley showed that the *normalized spatial curvature (form)* of a wavefield acts as *active information*. Dzog computes $Q$ on the GPU so AI craft and Chladni swarms naturally surf interference valleys without A* pathfinding. |
| **Kane S. Yee & Richard Courant** *(1966, 1928)* | **Staggered FDTD Lattices & The CFL Condition** | Naive grid solvers suffer from checkerboard artifacts and `NaN` explosions. Yee's staggered grid (pressure at cell centers, velocity on cell faces at half-time steps) and the Courant–Friedrichs–Lewy (CFL) time-step bound guarantee exact momentum conservation and numerical stability at $60\text{–}120\text{ Hz}$. |
| **Augustin-Jean Fresnel & Alan Griffith** *(1823, 1921)* | **Acoustic Impedance ($Z = \sqrt{\rho K}$) & Fracture Mechanics** | Replaces `if/else` collision scripts and integer hitpoints (`hp: 3`) with continuum mechanics. Impedance contrasts govern wave reflection and transmission automatically, while overpressure erodes material cohesion along stress planes until crystals cleave topologically. |
| **James Hillman** *(1977)* | **The Situated Image (*Inhesion*)** | Hillman distinguished between a detached *symbol* (a dictionary label like `hp: 2`) and a *situated image* whose behavior inheres in its environment. In Dzog, visuals, 3D crystal lighting, and modal audio are trustworthy projections of the same underlying physical field. |

---

## 2. De Broglie Nonlinear Wave Mechanics: Turning Waves into Soliton Projectiles

### 2.1 Overview for Computer Scientists
If you drop a pebble into a pond, the ripple spreads outward in a circle and fades away. In physics, this spreading is called *dispersion*. If a game engine eliminates rigid-body bullet structs so that a plasma bolt or a Pong ball is literally a wave traveling through the grid, standard linear wave math fails—your projectile spreads out into a faint ring before crossing the screen.

### 2.2 Why It Matters to the Engine (De Broglie's Double Solution)
In *Nonlinear Wave Mechanics: A Causal Interpretation* (1956), **Louis de Broglie** resolved this exact problem through his **Theory of the Double Solution**. Rather than treating a particle as a point mass separate from a wave, de Broglie modeled the physical wave $u(\mathbf{x}, t)$ as two coupled components:
1. **A Singular Soliton Core ($u_0$)**: A tiny region of intense energy where nonlinear self-focusing counteracts dispersion, holding the wave packet together like a solid particle.
2. **A Surrounding Pilot Wave ($v$)**: A broader, linear wavefield that radiates outward, senses environmental boundaries, and guides the core via de Broglie's **Law of Phase Harmony** (keeping the internal phase of the core locked to the phase of the surrounding wave).

In Dzog, every projectile (`SolitonBolt`, `DirectionalWavePacket`) is an executable implementation of de Broglie's double solution. Weapons punch a localized wave packet into the lattice whose nonlinear self-focusing holds it together across the arena, while its surrounding wake interacts with shields, walls, and other projectiles.

### 2.3 Technical Details: The Saturating Soliton & Kerr Self-Channeling Laws
Standard textbook equations for solitons (such as cubic Kerr nonlinearity, $\beta p^3$) have a fatal flaw for real-time software: when two large waves collide, the cubic term grows without bound (`NaN`), crashing the GPU pipeline.

Dzog solves this by combining a **bounded, rational nonlinear source term** $f_{\text{nl}}(p)$ and **de Broglie graded-index Kerr self-channeling** $K_{\text{eff}}(p)$ inside the compressible wave equations:

$$\rho(\mathbf{x}) \frac{\partial \mathbf{v}}{\partial t} = -\nabla p - \rho(\mathbf{x})\gamma(\mathbf{x})\mathbf{v}$$

$$\frac{\partial p}{\partial t} = -K_{\text{eff}}(\mathbf{x}, p)\nabla \cdot \mathbf{v} - \gamma(\mathbf{x})p + f_{\text{nl}}(p) + s(\mathbf{x}, t)$$

where the nonlinear self-focusing law and graded-index bulk modulus are defined as:

$$f_{\text{nl}}(p) = \frac{\beta \, p}{1 + \alpha \, p^2} - \kappa \, p, \qquad K_{\text{eff}}(\mathbf{x}, p) = K(\mathbf{x}) \max\!\left(0.35, \; 1 - \frac{\chi_{\text{lens}} \, p^2}{1 + \alpha \, p^2}\right)$$

Each parameter serves a specific systems engineering goal:

| Parameter | Plain Meaning | Role in the Runtime |
| :--- | :--- | :--- |
| **$\beta > 0$** | **Self-Focusing Gain** | Pulls high-energy wave peaks inward so projectiles hold a compact, particle-like shape across the screen. |
| **$\alpha > 0$** | **Safety Ceiling (Saturation)** | Caps maximum amplification at $\frac{\beta}{2\sqrt{\alpha}}$ as $p \to \infty$, preventing `NaN` blowups when shockwaves collide. |
| **$\kappa > 0$** | **Noise Gate (Vacuum Scrubbing)** | Dampens low-amplitude ripples below the soliton threshold so empty space stays clean and quiet. |
| **$\chi_{\text{lens}} \ge 0$** | **De Broglie Kerr Self-Channeling (`kerr_lensing`)** | Slightly slows phase speed inside high-amplitude cores ($c_{\text{eff}} \le c_{\text{max}}$), creating a convex graded-index waveguide that prevents 2D transverse spreading while strictly preserving the CFL stability bound. |
| **$\kappa_Q \ge 0$** | **Bohm–Hiley Pilot-Wave Guidance (`bohmian_guidance`)** | Steers soliton cores (`SolitonBolt::step_in_field`) and field-coupled actors (`reduce_body.wgsl`) along $-\nabla Q$ curvature valleys under de Broglie's Law of Phase Harmony. |

---

## 3. Bohm–Hiley Active Information ($Q$): Emergent Navigation Without Pathfinding

### 3.1 Overview for Computer Scientists
Normally, waves push objects through brute force: a taller ocean wave shoves a floating boat harder. In classical physics, force is proportional to the pressure gradient ($-\nabla p$), which dies off rapidly as a wave loses amplitude.

In their landmark work (*The Undivided Universe*, 1993; Hiley's *Process, Distinction, Groupoids and Clifford Algebras*, 2012), physicists **David Bohm** and **Basil Hiley** highlighted a fundamentally different mechanism by which waves organize matter: **Active Information**, mediated by the **Quantum Potential ($Q$)**. Unlike classical pressure, $Q$ does not depend on how *tall* a wave is; it depends on the **normalized spatial curvature (shape)** of the wave envelope.

Bohm and Hiley used the analogy of a ship on autopilot guided by radar waves: the radio signal does not physically push the ship's hull across the water with raw energy. Instead, the *form* of the low-energy radio wave informs the ship's higher-energy engines where to steer.

### 3.2 Why It Matters to the Engine
In traditional games, making an alien squadron fly in tight formations, orbit a carrier, or weave around incoming fire requires A* pathfinding graphs, steering raycasts, or hardcoded spline curves.

By computing Bohm and Hiley's Quantum Potential $Q$ directly on the GPU, **the wavefield itself becomes a dynamic navigation mesh**:
- **Chladni Swarm Formations**: A mothership (`ChladniCarrierEmitter`) radiates a faint, multi-lobed standing wave—analogous to the geometric sand patterns on a vibrating Chladni plate. Lightweight swarm drones (`BohmianSwarmDrone`) automatically slide into the wave's geometric curvature pockets ($Q < 0$) and surf alongside the carrier.
- **Automatic Obstacle Avoidance**: When an asteroid or enemy laser bolt enters the area, it bends and reflects the carrier wave. Because the drones follow the wave's curvature ($-\nabla Q$), the entire formation organically parts around obstacles and reforms on the other side with zero pathfinding code.

### 3.3 Technical Details: The Bohm–Hiley Quantum Potential & Dual Force Law
From the regularized wave amplitude envelope $R(\mathbf{x}, t) = \sqrt{p(\mathbf{x}, t)^2 + \epsilon^2}$, the engine computes the scalar Bohm–Hiley quantum potential $Q(\mathbf{x}, t)$:

$$Q(\mathbf{x}, t) = -\frac{\nabla^2 R(\mathbf{x}, t)}{R(\mathbf{x}, t)} \quad (\epsilon = 0.08)$$

1. **Scale-Invariance (Form over Amplitude)**:
   Because $R$ appears in both the numerator ($\nabla^2 R$, curvature) and the denominator ($R$, amplitude), multiplying the wave height by a constant does not change $Q$ outside the vacuum floor $\epsilon$. A subtle standing-wave channel exerts just as clear a geometric guidance signal as a massive shockwave.
2. **Numerical Stability (Why $Q$ Guides Actors Rather Than Face Velocities)**:
   Directly feeding $-\nabla Q = \nabla(\nabla^2 R / R)$ back into the Eulerian grid's face-velocity update would introduce an explicit 3rd-order spatial derivative, amplifying high-frequency ($k \sim \pi / \Delta x$) reflections at rigid walls. Instead, Dzog keeps the FDTD wave stencil strictly hyperbolic and evaluates $Q$ into the `State.a` texture channel (`velocity.wgsl`).
3. **The Unified Actor Guidance Law**:
   Every field-coupled body (`reduce_body.wgsl`) and de Broglie soliton core (`SolitonBolt::step_in_field`) integrates classical radiation pressure ($-\nabla p$, brute-force push), Bohmian pilot-wave guidance ($-\nabla Q$, geometric steering), and Poisson gravity ($-\nabla \Phi_g$):
   $$\mathbf{F}_{\text{total}} = -\kappa_p \nabla p - \kappa_Q \nabla Q - m \nabla \Phi_g$$
4. **Chladni Carrier Standing Waves (`ChladniCarrierEmitter`)**:
   Cruisers emit multi-lobed standing waves $p_{\text{carrier}}(r, \theta, t) = A (1 - r/R_0)^2 \cos(k_r r - \phi)\cos(m \theta)$. Swarm drones experience $-\nabla Q$ attraction into the nodal valleys ($Q < 0$), locking into geometric formations that dynamically flex under external wave perturbations.

---

## 4. Material Impedance, Topological Cleaving & Exciton Annealing

### 4.1 Overview for Computer Scientists
In standard games, physical objects are represented by an integer health bar (`hp: 3`) and a material tag (`enum Material { Metal, Rock }`). When `hp` reaches zero, the object is deleted and replaced by particle effects.

In the real world, objects do not have health bars. A block of piezoelectric quartz, a slab of volcanic obsidian, and a porous sponge react differently when struck because their internal **density** and **stiffness** differ. When a crystal shatters, it breaks along continuous crack lines where mechanical stress concentrates; when a crystal grows or heals, new structure anneals outward from the surviving core.

### 4.2 Why It Matters to the Engine
Instead of writing custom material scripts (`if target.type == Metal { reflect_bullet() }`) or tracking integer hitpoints, Dzog assigns every cell four continuum properties: **Density ($\rho$)**, **Bulk Modulus / Stiffness ($K$)**, **Damping ($\gamma$)**, and **Cohesion ($C \in [0, 1]$)**.
- **Zero-Script Material Reflections**: Wave speed and reflection ratios follow automatically from $\rho$ and $K$ via **Augustin-Jean Fresnel's** wave interface laws.
- **True Topological Fracture**: Lasers and shockwaves erode cohesion $C$ at the exact cells they strike. Partially eroded cells form internal waveguides that trap and channel subsequent shockwaves along the crack until the object splits topologically into independent tumbling fragments.
- **Tri-Lobe Ships & Reversible Annealing**: In the `shmup` demo, the player's ship consists of three physical crystal lobes (Port Wing, Core, Starboard Wing) rather than hitpoints. Taking a heavy hit on the left wing physically severs that wing, eliminating its thruster and weapon mount; catching a field-guided piezoelectric "Power Cube" triggers **exciton annealing** ($\Delta C > 0$), regrowing the lost crystal wing cell by cell.

### 4.3 Technical Details: Fresnel Impedance, Griffith Fracture & Antiphase Harvesting

```text
   [ Constitutive Material Grid: ρ(x), K(x), γ(x), C(x) ]
          ^                                      |
          | Erosion (ΔC < 0) / Annealing (ΔC > 0)| Fresnel Impedance Z = √(ρK)
          | Moving Hull Work (v_n = v_actor)     | & Wave Speed c = √(K/ρ)
          |                                      v
   [ Situated Actors ] <---------------- [ Active Field: p, v, Q, L ]
                        Guidance: F = -κ_p ∇p - κ_Q ∇Q - m ∇Φ_g
```

1. **Acoustic Impedance & Fresnel Reflections**:
   Each cell defines a local wave speed $c(\mathbf{x}) = \sqrt{K(\mathbf{x})/\rho(\mathbf{x})}$ and characteristic acoustic impedance $Z(\mathbf{x}) = \sqrt{\rho(\mathbf{x}) K(\mathbf{x})}$. When a wave crosses from medium $Z_1$ into $Z_2$, reflection ($R$) and transmission ($T$) follow Fresnel's equations automatically from the staggered grid updates:
   $$R = \frac{Z_2 - Z_1}{Z_2 + Z_1}, \qquad T = \frac{2 Z_2}{Z_2 + Z_1}$$
   Dense `ObsidianAnvil` ($Z = 2.02$) naturally reflects glancing laser bolts (`CollimatedLaserBeam::fire_into_field`), while `PiezoQuartz` ($c = 1.48$) transmits fast internal stress waves.
2. **Continuous Cohesion Erosion, BFS Cleaving & Annealing**:
   Inspired by **Alan Griffith's** fracture mechanics, when local pressure $|p|$ exceeds a material's yield threshold $p_{\text{yield}}$, cohesion erodes proportionally to the overpressure:
   $$C^{n+1} = \max\!\left(0, \; C^n - \eta_{\text{erode}}\left(|p| - p_{\text{yield}}\right)\Delta t\right)$$
   Partially eroded cells ($0.15 \le C < 0.55$) have reduced stiffness $K$, acting as internal waveguides that concentrate stress along existing cracks. Once a fracture line reaches $C = 0$ across a body, a 4-connected **Breadth-First Search (BFS)** pass (`fracture.rs`) detects the topological split, spawning independent child bodies that conserve linear and angular momentum. Vessel mortality is evaluated directly from lattice topology (`largest_intact_component_ratio()`, `structural_integrity()`, `core_cohesion()`). Conversely, absorbing an annealing condensate (`CohesiveBody::anneal_cells`) drives $\Delta C > 0$ outward from intact core cells, sealing fractures and re-crystallizing severed ship lobes.
3. **Moving Boundaries & Antiphase Energy Harvesting**:
   Moving hulls stamp their instantaneous velocity ($v_n = v_{\text{actor}}$) into cell-face masks, compressing waves ahead of them (Doppler shift). The lattice tracks total **Hamiltonian mechanical energy** ($H = E_p + E_k$) across potential and kinetic components:
   $$E_p = \sum \frac{p^2}{2K} \Delta A, \qquad E_k = \sum \frac{1}{2}\rho |\mathbf{v}|^2 \Delta A$$
   When the player fires an **Antiphase Shield** (`AntiphaseShieldPulse`), it injects an inverted wavefront ($\Delta \phi = \pi$) that destructively cancels incoming enemy fire. The engine measures the resulting drop in local Hamiltonian energy ($\Delta H = E_{\text{before}} - E_{\text{after}}$) and harvests that annihilated wave energy directly into the ship's weapon capacitor.

---

## 5. Real-Time Numerical Engineering & Zero-Readback GPU Architecture

### 5.1 Overview for Computer Scientists
Simulating partial differential equations (PDEs) across tens of thousands of cells at 60 to 120 frames per second is computationally demanding. If you discretize a wave equation naively on a standard grid, you encounter three classic pitfalls:
1. **Checkerboard Artifacts**: Storing pressure and velocity at the exact same grid points causes adjacent cells to decouple into alternating high/low checkerboards.
2. **Numerical Explosions (`NaN`)**: If a wave travels faster than one grid cell per time step, or if damping subtracts too much energy in a single frame, floating-point values oscillate to infinity.
3. **PCIe Bus Stalls**: Copying a full-screen GPU texture back to the CPU every frame just to calculate forces on 50 objects stalls the graphics pipeline.

### 5.2 Why It Matters to the Engine
For a field engine to power fast-paced interactive games, it must guarantee:
- **Unconditional Stability**: The solver must never produce `NaN` values, even during chaotic multi-ship battles with overlapping shockwaves.
- **Zero-Readback GPU Speed**: The entire wave step and body force integration must stay on the GPU, sending only a tiny summary struct (forces, torques, and audio samples) back to the CPU.
- **Deterministic Counterfactual Branching**: Because the entire causal state of the world lives in uniform lattice buffers, players and developers can snapshot any moment (`[B]`), test a counterfactual action, and restore the branch (`[U]`) with bit-exact reproducibility on a given backend.

### 5.3 Technical Details: Yee Staggering, CFL Bounds & WGPU Compute Pipeline

#### Staggered Grid Discretization & Stability Laws
Building on **Kane S. Yee's** (1966) staggered lattice and **Courant, Friedrichs, and Lewy's** (1928) stability criterion, `dzog-core` and `dzog-wgpu` enforce three numerical invariants:
- **Yee Staggered Placement**: Pressure $p$, Bohmian $Q$, luminescence $L$, and material properties $(\rho, K, \gamma, C)$ live at **cell centers** $(i+\frac{1}{2}, j+\frac{1}{2})$ at integer steps $t^n$. Velocities $v_x, v_y$ and moving wall masks live on **East/North cell faces** at half-steps $t^{n+1/2}$. Storing pressure and velocity half a cell apart eliminates odd-even decoupling and guarantees exact momentum conservation across moving boundaries.
- **Global CFL Time-Step Bound**: Before stepping, `SolverConfig::compute_cfl_dt` locks $\Delta t$ against the fastest wave speed across all materials on the grid:
  $$c_{\text{max}} = \sqrt{\frac{\max K}{\min \rho}}, \qquad \Delta t \le \frac{\text{safety}}{c_{\text{max}} \sqrt{\Delta x^{-2} + \Delta y^{-2}}}$$
- **Unconditionally Positive Exponential Damping**: Energy dissipation integrates via $\exp(-\gamma \Delta t)$ rather than Euler subtraction $(1 - \gamma \Delta t)$, preventing sign-flip instabilities inside high-absorption boundary sponges and fractured crystal veins.

#### Zero-Readback WGPU Pipeline & Texture Layout (`dzog-wgpu`)

```text
FieldSpec / GPU Impulse Shader (impulse.wgsl) + SolitonCarrierPulse
       |
       v
WGPU Compute Substeps (Pass 1: Velocity + Bohmian Q -> Pass 2: Pressure + Soliton -> Pass 3: Luminescence L)
       |                                                        |
       v                                                        v
Zero-Readback Body Force/Probe Reduction (reduce_body.wgsl)   Bevy Field Canvas & Perceptual Views
       |
       v
48 kHz Modal Synthesizer (FieldModalSynthesizer -> bevy_audio Decodable)
```

Field states are packed into cache-coherent `RGBA32Float` ping-pong textures:

| GPU Texture | Format | Packed Channels | Physical Role |
| :--- | :--- | :--- | :--- |
| `State A / B` | `RGBA32Float` | `(p_center, vx_east, vy_north, q_bohmian)` | Active wave pressure, staggered face velocities, and Bohm–Hiley quantum potential $Q$. |
| `Luminescence A / B` | `RGBA32Float` | `(luminescence_L, 0, 0, 0)` | Reaction-diffusion exciton glow field $L(\mathbf{x}, t)$ for 3D crystal lighting and smoke/fire decay. |
| `Material` | `RGBA32Float` | `(density_rho, bulk_modulus_K, damping_gamma, cohesion_C)` | Constitutive material parameters and topological structural cohesion $C \in [0, 1]$. |
| `Boundary` | `RGBA32Float` | `(wall_east_mask, wall_vx, wall_north_mask, wall_vy)` | Moving hull face masks and instantaneous Doppler boundary velocities ($v_n = v_{\text{actor}}$). |

Each macro-frame executes five WGSL compute stages:
1. **`impulse.wgsl`**: Stamps moving hulls, thruster plumes, and phase-locked carrier pulses into the grid.
2. **`velocity.wgsl`**: Updates staggered face velocities $v_x, v_y$ and evaluates the Bohmian potential $Q$.
3. **`pressure.wgsl`**: Updates cell-center pressure $p$, applies the rational soliton law $f_{\text{nl}}(p)$ and Kerr lensing $K_{\text{eff}}(p)$, and erodes cohesion $C$.
4. **`luminescence.wgsl`**: Evolves the reaction-diffusion exciton glow field $L$.
5. **`reduce_body.wgsl`**: Performs a GPU parallel reduction over each body's bounding contour, returning a compact storage buffer of net forces ($-\nabla p, -\nabla Q$), recoil torques ($\mathbf{r} \times \mathbf{J}$), and acoustic probe samples (up to 128 bodies) with zero texture readback.

---

## 6. Perceptual Projections: Hillman's Situated Image in Sight & Sound

### 6.1 Overview for Computer Scientists
In conventional engines, graphics, lighting, and sound are separate facade layers bolted onto game state: hitting an asteroid decrements `hp`, plays a pre-recorded `explosion.wav` file, and spawns a particle sprite.

In *An Inquiry into Image* (1977), psychologist and philosopher **James Hillman** drew a sharp distinction between a **symbol** and a **situated image**. A symbol reduces an object to an isolated, context-free label; an *image* retains its full ecological context, mood, topology, and internal relations (*inhesion*). In Dzog, visuals, 3D crystal lighting, diagnostic lenses, and audio are not separate asset pipelines—they are direct, trustworthy projections of the same evolving physical field.

### 6.2 Why It Matters to the Engine
Because the lattice already computes physical pressure oscillations, exciton glow diffusion, and material wave speeds:
- **Field-Driven 3D Crystal Lighting**: Passing laser bolts and stressed crystal cracks emit a persistent luminescence field ($L$) that diffuses through space and dynamically illuminates the 3D facets of nearby rotating asteroids.
- **Multiple Diagnostic Lenses**: Developers and players can switch between the composite game view and diagnostic views (inspecting pilot-wave channels $Q$, Poynting energy flux $\mathbf{I}$, or material stress $C$) at any time without altering the underlying physics.
- **Zero `.wav` Files Required**: Virtual microphones attached to the player's ship sample the exact pressure waves hitting the hull and drive a real-time $48\text{ kHz}$ modal synthesizer tuned to the mineral stiffness of the struck object.

### 6.3 Technical Details: Exciton Luminescence, Multi-View Rendering & Modal Audio
1. **Reaction-Diffusion Exciton Luminescence ($L$)**:
   Pass 3 (`luminescence.wgsl`) evolves a radiative light field $L(\mathbf{x}, t) \in [0, 4]$ driven by wave energy $S_{\text{wave}}(p, \mathbf{v})$ and piezoelectric crystal stress $S_{\text{piezo}}(p, C)$, diffusing at a rate $D \propto c(\mathbf{x})$:
   $$L^{n+1} = e^{-\gamma_L \Delta t}\left(L^n + D \Delta t \, \nabla^2 L^n\right) + \Delta t \left(S_{\text{wave}} + S_{\text{piezo}}\right)$$
   Spatial gradients $\nabla L$ and $\nabla p$ compute per-pixel 3D facet normals on rotating mineral hulls (`CohesiveBody::sample_crystal_facet`), so glowing cracks and passing solitons cast directional specular light onto nearby bodies before cooling ($e^{-1.85 \Delta t}$) into dark condensing smoke.
2. **Perceptual View Projections (`FieldViewMode`)**:
   - *Composite Experience View*: Renders 3D-faceted crystalline bodies lit by $\nabla L$ and $\nabla p$.
   - *Bohmian & Pressure View ($p, Q$)*: Exposes the pilot-wave guidance valleys steering AI trajectories.
   - *Velocity & Poynting Flux View ($\mathbf{v}, \mathbf{I} = p\mathbf{v}$)*: Visualizes instantaneous energy transport vectors.
   - *Impedance & Cohesion View ($Z, C$)*: Displays material commitments and internal fracture waveguides.
3. **Material-Tuned Modal Audio (`FieldModalSynthesizer`)**:
   Detailed in [`docs/overview_audio.md`](./overview_audio.md), `BinauralCockpitSonifier` samples port and starboard wingtip pressures and Doppler closing speeds at $60\text{ Hz}$, feeding a lock-free $48\text{ kHz}$ Bevy `Decodable` stream (`DzogFieldAudioSource`). Strike frequencies derive directly from the struck material's acoustic wave speed:
   $$f_0 = 440 \, c(\mathbf{x}) = 440 \sqrt{\frac{K(\mathbf{x})}{\rho(\mathbf{x})}}$$
   High-stiffness `PiezoQuartz` rings with bright crystalline harmonics, while dense `ObsidianAnvil` produces low metallic clangs. Sub-threshold vacuum ripples ($|p| < 0.15$) are gated to exact `(0.0, 0.0)` silence.

---

## 7. Deep-Dive Q&A: How Projectiles & Combat Work in a Pure Field

> **Reader Question:** *"The bullets in your shmup—are those just treated as particles in a pilot-wave model or something?"*

**Short Answer:** No. Projectiles are not passive point particles riding a separate pilot wave. In **Dzog**, weapons fire **de Broglie solitons**—self-focusing wave packets coupled directly to the GPU wave-matter lattice.

### 7.1 Mental Model: Isolated Messengers vs. The Nonlinear Trampoline
- **Traditional Engine (Isolated Messengers):** Space is an empty coordinate void. Firing 50 bullets spawns 50 isolated structs that do not interact with the space between them; the CPU checks bounding boxes every frame and decrements integer hitpoints (`hp -= 1`) on contact.
- **Dzog (The Nonlinear Trampoline):** The arena is a single taut trampoline mat of interconnected GPU cells (`240×135` `16:9` lattice). Firing a weapon punches the mat in a specific direction (`DirectionalWavePacket` + `SolitonCarrierPulse`), sending a tight, self-contained **bump (a soliton)** across the fabric. Because the wave equation includes the nonlinear self-focusing law $f_{\text{nl}}(p)$ and graded-index Kerr lensing $K_{\text{eff}}(p)$, high-amplitude peaks pull themselves inward rather than flattening out like linear pond ripples.

### 7.2 Unifying Wave Excitation and Guidance in Combat
Under de Broglie's *Double Solution* ($u = u_0 + v$) and Bohm–Hiley hydrodynamics, `dzog-core` (`ordnance.rs`, `solver.rs`) unifies projectile motion, beam optics, and structural combat:
- **Phase-Locked Soliton Bolts (`SolitonBolt::step_in_field`)**: To allow high-speed ordnance to cross a `240×135` lattice without Nyquist grid dispersion, each bolt advances its singular core $u_0$ under de Broglie's *Law of Phase Harmony*—sampling local Bohmian quantum force $-\nabla Q$ and medium wave speed $c(\mathbf{x})$ from the surrounding wave $v$, while `couple_carrier_to_lattice` phase-locks its $(p, v_x, v_y, L)$ envelope into the grid.
- **Fresnel Laser Beams (`CollimatedLaserBeam::fire_into_field`)**: Continuous beams propagate self-channeled wave energy along a beam waist, computing exact Fresnel reflection and transmission ($R = \frac{Z_2 - Z_1}{Z_2 + Z_1}$) at crystal facets and spawning reflected wave packets off high-impedance armor (`ObsidianAnvil`).
- **Bullet Hell as Wave Interference**: Crossing wave wakes superpose in the `State` texture into constructive peaks or destructive troughs, creating $-\nabla p$ radiation-pressure wakes and $-\nabla Q$ guidance ridges that buffet nearby ships, shards, and annealing `PowerCube`s.

### 7.3 Summary: What Lives in the Lattice During a Barrage

| Lattice Channel | Trampoline Analogy | Role During a Projectile Barrage |
| :--- | :--- | :--- |
| **Pressure ($p$)** *(`State.r`, cell center)* | How high or low this exact spot on the mat is pushed right now. | Stores active wave amplitude; projectile peaks are held compact against dispersion by the rational soliton law $f_{\text{nl}}(p)$, Kerr lensing $K_{\text{eff}}(p)$, and `SolitonCarrierPulse` injection. |
| **Velocity ($v_x, v_y$)** *(`State.gb`, staggered East/North faces)* | Which way tension pulls between adjacent squares. | Encodes the directional momentum and Poynting energy flux ($\mathbf{I} = p\mathbf{v}$) of each traveling wave packet. |
| **Bohmian Potential ($Q$)** *(`State.a`, curvature $-\nabla^2 R / R$)* | The normalized slope and curvature geometry stretching around wave peaks ($R = \sqrt{p^2 + \epsilon^2}$). | Acts as an early-warning guidance field, steering `SolitonBolt` phase harmony and allowing AI craft and Chladni swarms to sense incoming salvos and surf quiet nodal valleys. |
| **Luminescence ($L$)** *(`Luminescence.r`, cell center)* | How brightly the mat glows or smolders where energy concentrates. | Driven by wave energy ($|p| > 0.36$) and crystal fracture stress ($C \ge 0.15, |p| > 0.38$), diffusing and cooling to illuminate 3D crystal facets and render fiery explosions that condense into dark smoke. |
| **Material ($\rho, K, \gamma, C$)** *(`Material.rgba`, cell center)* | Whether a cell is open vacuum ($C=0$) or a solid crystalline hull/asteroid ($C \in (0, 1]$). | Projectiles travel through open space ($C=0$) and transfer momentum into high-impedance hulls ($Z = \sqrt{\rho K}$), eroding cohesion $C \to 0$ along stress planes until dynamic connectivity drops below survival thresholds. |

---

## 8. Engineering Boundaries & References

### 8.1 Engineering Boundaries

| Capability | Engineering Boundary |
| :--- | :--- |
| **GPU Stencil Parallelism** | $O(\text{cells} \times \text{substeps})$ uniform dispatch work; active vacuum computes potentials across all cells every step. |
| **Collision-Free Wave Agency** | Wave interference, radiation pressure, and reflection require zero BVH queries, complemented by compact local-grid body contours. |
| **Discrete Continuum Limits** | Finite-difference lattices remain bounded by declared CFL time steps and Nyquist spatial wavelengths. |
| **Cross-Device Reproducibility** | Bit-exact within a backend; CPU (`dzog-core`) and GPU (`dzog-wgpu`) floating-point stencils agree within verified tolerances ($< 10^{-4}$). |

### 8.2 Foundational References

1. **Louis de Broglie** (1956), *Nonlinear Wave Mechanics: A Causal Interpretation* (Elsevier) & (1927), *La mécanique ondulatoire et la structure atomique de la matière et du rayonnement* — Theory of the Double Solution ($u = u_0 + v$), soliton self-focusing, and the Law of Phase Harmony.
2. **David Bohm & Basil Hiley** (1993), *The Undivided Universe: An Ontological Interpretation of Quantum Theory* (Routledge) — The Quantum Potential ($Q = -\nabla^2 R / R$), active information, and pilot-wave guidance.
3. **Basil J. Hiley** (2012), [*Process, Distinction, Groupoids and Clifford Algebras*](https://arxiv.org/abs/1211.2107) — Process-first algebraic formulation of quantum field mechanics, demonstrating how objects emerge as invariant persistent processes (*idempotents*) within an underlying activity.
4. **Kane S. Yee** (1966), *Numerical solution of initial boundary value problems involving Maxwell's equations in isotropic media* (IEEE Trans. Antennas Propag.) & **Courant, Friedrichs, and Lewy** (1928) — Staggered finite-difference time-domain (FDTD) conservation and hyperbolic time-step stability bounds.
5. **Augustin-Jean Fresnel** (1823) & **Alan A. Griffith** (1921), *The Phenomena of Rupture and Flow in Solids* — Acoustic impedance reflection/transmission laws and stress-concentrated fracture propagation.
6. **James Hillman** (1977), *An Inquiry into Image* (Spring Publications) — The particularity and *inhesion* of the situated image versus detached symbolic labels.
