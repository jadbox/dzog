//! Late de Broglie Double-Solution ($u = u_0 + v$) Solitons, Bohmian Guided Missiles,
//! and Fresnel-Refracting Collimated Laser Beams.
//!
//! Operationalizes `philosophy.md` and `docs/field_first_whitepaper.md`:
//! - **Double-Solution Solitons (`SolitonBolt`)**: Rather than an isolated rigid-body point particle,
//!   each projectile is a high-amplitude nonlinear core $u_0$ phase-locked to the surrounding linear
//!   pilot wave $v(\mathbf{x}, t)$. Its trajectory obeys de Broglie's *Law of Phase Harmony*, bending
//!   with local Bohm-Hiley quantum potential gradients ($-\nabla Q$), radiation pressure ($-\nabla p$),
//!   and medium wave speed $c(\mathbf{x}) = \sqrt{K_{\text{eff}}/\rho}$.
//! - **Field-Mediated Antiphase Annihilation**: When an `AntiphaseShieldPulse` or destructive wave
//!   interference collapses the lattice energy envelope $R(\mathbf{x}, t)$ at the soliton's core,
//!   the soliton's amplitude attenuates and dissolves back into the continuum.
//! - **Collimated Laser Beams (`CollimatedLaserBeam`)**: Self-channeling high-frequency wave beams
//!   that propagate across the lattice, refract/reflect at crystal impedance boundaries via Fresnel's
//!   laws ($R = (Z_2 - Z_1)/(Z_2 + Z_1)$), and erode structural cohesion $C(\mathbf{x})$ at the focal spot.

use serde::{Deserialize, Serialize};
use crate::body::CohesiveBody;
use crate::excitation::{DirectionalWavePacket, GaussianImpulse, SolitonCarrierPulse};
use crate::grid::AcousticState;
use crate::material::MaterialField;

/// Ordnance classification for field-coupled wave-packet projectiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileKind {
    /// High-velocity kinetic-soliton bullet that imparts strong linear & angular recoil (`r x J`)
    /// and progressive fractures to airborne hulls and asteroids.
    KineticBullet,
    /// Coherent plasma-soliton wave packet (default in `asteroids`).
    PlasmaSoliton,
    /// Alt-fire Air-to-Ground finned missile that flies beneath sky squadrons and steers down
    /// Bohmian quantum potential valleys (`-grad Q`) and topographic gradients toward ground structures.
    AirToGroundMissile,
    /// Enemy plasma-soliton orb.
    EnemyOrb,
}

/// Coherent double-solution ($u = u_0 + v$) soliton projectile coupled bidirectionally to the lattice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolitonBolt {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub ttl: u16,
    pub from_player: bool,
    pub overcharged: bool,
    pub kind: ProjectileKind,
}

impl SolitonBolt {
    /// Construct a player or enemy plasma-soliton bolt (`ProjectileKind::PlasmaSoliton` / `EnemyOrb`).
    #[allow(clippy::too_many_arguments)]
    pub fn new_plasma(
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        radius: f32,
        ttl: u16,
        from_player: bool,
        overcharged: bool,
    ) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            radius,
            ttl,
            from_player,
            overcharged,
            kind: if from_player {
                ProjectileKind::PlasmaSoliton
            } else {
                ProjectileKind::EnemyOrb
            },
        }
    }

    /// Advance this soliton core $u_0$ by `dt_sub` under de Broglie Phase Harmony and Bohm-Hiley
    /// active information (`-grad Q`), then phase-lock its carrier wave into the lattice.
    ///
    /// Returns `false` if `ttl` expired or if the bolt moved outside a non-periodic arena.
    pub fn step_in_field(
        &mut self,
        state: &mut AcousticState,
        material: &MaterialField,
        dt_sub: f32,
        periodic: bool,
        wave_force_enabled: bool,
        homing_targets: &[(f32, f32)],
    ) -> bool {
        if self.ttl == 0 {
            return false;
        }

        let w = state.config.width;
        let h = state.config.height;
        let wf = w as f32;
        let hf = h as f32;

        // 1. Bohm-Hiley Active Information & Target Wave-Homing for Guided Missiles
        if self.kind == ProjectileKind::AirToGroundMissile {
            if let Some(&(tx, ty)) = homing_targets
                .iter()
                .filter(|&&(gx, gy)| gy > 8.0 && gy < self.y + 16.0 && (gx - self.x).abs() < 96.0)
                .min_by(|a, b| {
                    let da = (a.0 - self.x).powi(2) + (a.1 - self.y).powi(2);
                    let db = (b.0 - self.x).powi(2) + (b.1 - self.y).powi(2);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                let dx = tx - self.x;
                let dy = (ty - self.y).min(-2.0);
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let desired_vx = (dx / dist) * 1.45;
                self.vx = (self.vx * 0.86 + desired_vx * 0.14).clamp(-1.25, 1.25);
            }
        }

        // 2. De Broglie Phase Harmony: sample local Bohmian quantum force F_Q = -grad(Q)
        //    and refractive medium impedance so passing solitons bend through standing-wave channels.
        if wave_force_enabled {
            let gx = (self.x.round() as isize).rem_euclid(w as isize) as usize;
            let gy = (self.y.round() as isize).rem_euclid(h as isize) as usize;
            if periodic || (gx >= 2 && gx + 2 < w && gy >= 2 && gy + 2 < h) {
                let (q_fx, q_fy) = state.quantum_force_wrapped_at(gx, gy, periodic);
                let c_local = material.wave_speed_at(gx, gy).clamp(0.75, 1.35);
                let q_gain = match self.kind {
                    ProjectileKind::AirToGroundMissile => 0.012 * c_local,
                    ProjectileKind::EnemyOrb => 0.006 * c_local,
                    ProjectileKind::KineticBullet | ProjectileKind::PlasmaSoliton => 0.004 * c_local,
                };
                self.vx += (q_fx * q_gain).clamp(-0.04, 0.04) * dt_sub;
                self.vy += (q_fy * q_gain).clamp(-0.04, 0.04) * dt_sub;
            }
        }

        // 3. Advance core coordinates
        self.x += self.vx * dt_sub;
        self.y += self.vy * dt_sub;

        if periodic {
            self.x = self.x.rem_euclid(wf);
            self.y = self.y.rem_euclid(hf);
        } else if self.x < 2.0 || self.x >= wf - 2.0 || self.y < 2.0 || self.y >= hf - 2.0 {
            return false;
        }

        true
    }

    /// Couple the traveling soliton core $u_0$ into the staggered lattice $(p, v_x, v_y, L)$.
    pub fn couple_carrier_to_lattice(
        &self,
        state: &mut AcousticState,
        material: &MaterialField,
        periodic: bool,
    ) {
        let (sigma, amp, lum_gain) = match self.kind {
            ProjectileKind::PlasmaSoliton => {
                if self.overcharged {
                    (1.55, 1.35, 0.95)
                } else {
                    (1.20, 0.82, 0.65)
                }
            }
            ProjectileKind::KineticBullet => {
                if self.overcharged {
                    (1.25, 0.55, 0.35)
                } else {
                    (0.95, 0.24, 0.12)
                }
            }
            ProjectileKind::AirToGroundMissile => (1.10, 0.32, 0.16),
            ProjectileKind::EnemyOrb => {
                if periodic {
                    (1.20, 0.78, 0.60)
                } else {
                    (1.05, 0.28, 0.14)
                }
            }
        };
        SolitonCarrierPulse::new(self.x, self.y, self.vx, self.vy, sigma, amp, lum_gain)
            .couple_to_lattice(state, material, periodic);
    }
}

/// Standardized impact record when a field-coupled projectile strikes a `CohesiveBody`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrdnanceImpact {
    pub target_idx: usize,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub overcharged: bool,
    pub kind: ProjectileKind,
}

/// Result of a collimated laser beam interacting with the field and solid-state crystal hulls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaserStrike {
    pub hit_x: f32,
    pub hit_y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub body_index: Option<usize>,
    pub reflection_coeff: f32,
    pub transmitted_energy: f32,
}

/// Continuous Self-Channeling Collimated Laser Beam (`philosophy.md` §Wave-Mediated Agency).
///
/// Excites a collimated, phase-coherent wave channel along `angle_rad` from `(origin_x, origin_y)`.
/// When the beam encounters a solid material interface ($Z_2 = \sqrt{\rho_2 K_2}$) or `CohesiveBody`:
/// 1. Computes exact Fresnel reflection ($R = (Z_2 - Z_1)/(Z_2 + Z_1)$) and transmission ($T = 1 - R^2$).
/// 2. Spawns a reflected wave packet off high-impedance armor (`ObsidianAnvil` / crystal facets).
/// 3. Transfers radiation pressure impulse ($-\nabla p$) and carves thermal/piezo fracture fissures
///    into the struck body.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CollimatedLaserBeam {
    pub origin_x: f32,
    pub origin_y: f32,
    pub angle_rad: f32,
    pub max_range: f32,
    pub beam_waist: f32,
    pub intensity: f32,
}

impl CollimatedLaserBeam {
    pub fn new(
        origin_x: f32,
        origin_y: f32,
        angle_rad: f32,
        max_range: f32,
        beam_waist: f32,
        intensity: f32,
    ) -> Self {
        Self {
            origin_x,
            origin_y,
            angle_rad,
            max_range: max_range.max(2.0),
            beam_waist: beam_waist.max(0.6),
            intensity: intensity.max(0.0),
        }
    }

    /// Propagate the collimated laser beam through the lattice and against `bodies`,
    /// injecting self-channeled wave energy, Fresnel reflections, and crystal cohesion erosion.
    pub fn fire_into_field(
        &self,
        state: &mut AcousticState,
        material: &mut MaterialField,
        bodies: &mut [CohesiveBody],
        periodic: bool,
    ) -> LaserStrike {
        let w = state.config.width as isize;
        let h = state.config.height as isize;
        let periodic_dims = if periodic {
            Some((w as f32, h as f32))
        } else {
            None
        };

        let dir_x = self.angle_rad.cos();
        let dir_y = self.angle_rad.sin();
        let perp_x = -dir_y;
        let perp_y = dir_x;

        let step_ds = 0.85f32;
        let max_steps = (self.max_range / step_ds).ceil() as usize;
        let z_vacuum = 1.0f32;
        let has_lum = state.luminescence.len() == (w as usize) * (h as usize);
        let has_coh = material.cohesion.len() == (w as usize) * (h as usize);

        let mut hit_x = self.origin_x + dir_x * self.max_range;
        let mut hit_y = self.origin_y + dir_y * self.max_range;
        let mut struck_body: Option<usize> = None;
        let mut reflection_coeff = 0.0f32;
        let mut transmitted_energy = self.intensity;

        for s in 0..max_steps {
            let dist = s as f32 * step_ds;
            let mut bx = self.origin_x + dir_x * dist;
            let mut by = self.origin_y + dir_y * dist;

            if periodic {
                bx = bx.rem_euclid(w as f32);
                by = by.rem_euclid(h as f32);
            } else if bx < 1.5 || bx >= w as f32 - 1.5 || by < 1.5 || by >= h as f32 - 1.5 {
                hit_x = bx.clamp(1.0, w as f32 - 2.0);
                hit_y = by.clamp(1.0, h as f32 - 2.0);
                break;
            }

            let gx = (bx.round() as isize).rem_euclid(w) as usize;
            let gy = (by.round() as isize).rem_euclid(h) as usize;
            let c_idx = state.config.index(gx, gy);

            // Inject collimated self-trapped wave core across the narrow beam waist
            for &lat in &[-1.0f32, 0.0, 1.0] {
                let px = (bx + perp_x * lat * self.beam_waist).round() as isize;
                let py = (by + perp_y * lat * self.beam_waist).round() as isize;
                if !periodic && (px < 0 || px >= w || py < 0 || py >= h) {
                    continue;
                }
                let ix = px.rem_euclid(w) as usize;
                let iy = py.rem_euclid(h) as usize;
                let idx = state.config.index(ix, iy);
                let env = (-0.5 * lat * lat).exp();
                let p_beam = self.intensity * 0.42 * env;
                state.pressure[idx] = (state.pressure[idx] + p_beam).clamp(-4.0, 4.0);
                let z_local = (material.density[idx] * material.bulk_modulus[idx])
                    .sqrt()
                    .max(0.1);
                state.vx[idx] += (p_beam / z_local) * dir_x * 0.5;
                state.vy[idx] += (p_beam / z_local) * dir_y * 0.5;
                if has_lum {
                    state.luminescence[idx] =
                        (state.luminescence[idx] + self.intensity * 0.48 * env).min(4.0);
                }
            }

            // Check if the laser beam strikes any rotating CohesiveBody contour
            for (b_idx, body) in bodies.iter_mut().enumerate() {
                let (lx, ly) = body.world_to_local(bx, by, periodic_dims);
                let coh = body.sample_local_cohesion(lx, ly);
                if coh >= 0.25 {
                    hit_x = bx;
                    hit_y = by;
                    struck_body = Some(b_idx);

                    let z_body = body.material.impedance();
                    let r_fresnel = ((z_body - z_vacuum) / (z_body + z_vacuum)).abs();
                    reflection_coeff = r_fresnel;
                    transmitted_energy = self.intensity * (1.0 - r_fresnel * r_fresnel);

                    // Impart radiation pressure momentum & torque (r x J)
                    let impulse_mag = self.intensity * (1.0 + r_fresnel) * 0.35;
                    body.apply_contact_impulse(
                        bx,
                        by,
                        dir_x * impulse_mag,
                        dir_y * impulse_mag,
                        periodic_dims,
                    );

                    // Carve thermal/piezo fracture waveguide at the focal spot
                    body.carve_cleavage_fissure(
                        bx,
                        by,
                        dir_x,
                        dir_y,
                        transmitted_energy * 0.85,
                        periodic_dims,
                    );

                    // Spawn Fresnel-reflected wave packet off the crystal facet normal
                    let (_, nx, ny, _) = body.sample_crystal_facet(lx, ly);
                    let n_len = (nx * nx + ny * ny).sqrt().max(0.001);
                    let unx = nx / n_len;
                    let uny = ny / n_len;
                    let dot = dir_x * unx + dir_y * uny;
                    let ref_dx = dir_x - 2.0 * dot * unx;
                    let ref_dy = dir_y - 2.0 * dot * uny;
                    let ref_angle = ref_dy.atan2(ref_dx);

                    let mut ref_pkt = DirectionalWavePacket::new(
                        bx - dir_x * 1.2,
                        by - dir_y * 1.2,
                        1.3,
                        (self.intensity * (0.45 + r_fresnel)).clamp(0.4, 2.4),
                        ref_angle,
                    );
                    ref_pkt.inject_wrapped(state, material, periodic);
                    break;
                }
            }

            if struck_body.is_some() {
                break;
            }

            // Check if the laser hits a static solid wall or high-cohesion terrain cell in the lattice
            if has_coh && material.cohesion[c_idx] >= 0.45 {
                hit_x = bx;
                hit_y = by;
                let z_wall = material.impedance_at(gx, gy);
                let r_fresnel = ((z_wall - z_vacuum) / (z_wall + z_vacuum)).abs();
                reflection_coeff = r_fresnel;
                transmitted_energy = self.intensity * (1.0 - r_fresnel * r_fresnel);
                material.cohesion[c_idx] =
                    (material.cohesion[c_idx] - transmitted_energy * 0.22).max(0.0);
                GaussianImpulse::new(bx, by, 1.5, self.intensity * 0.8)
                    .inject_wrapped(state, periodic);
                break;
            }
        }

        LaserStrike {
            hit_x,
            hit_y,
            dir_x,
            dir_y,
            body_index: struck_body,
            reflection_coeff,
            transmitted_energy,
        }
    }
}
