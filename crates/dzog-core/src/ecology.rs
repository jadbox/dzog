//! Hard-Body Field Ecology, Bidirectional Lattice Cohesion Sync, and Elastic/Fracture Contact.
//!
//! Moves shared continuum-to-body physics into `dzog-core` so game crates (`asteroids`, `shmup`)
//! remain focused on scenario composition rather than duplicating physical contact, kinematics,
//! or lattice cohesion erosion loops.

use std::f32::consts::TAU;
use crate::body::CohesiveBody;
use crate::excitation::GaussianImpulse;
use crate::grid::AcousticState;
use crate::material::MaterialField;

impl CohesiveBody {
    /// Synchronize overpressure cohesion erosion from the world lattice (`material.cohesion` and
    /// `state.pressure`) back into this body's rotating local crystal grid `self.cohesion`.
    ///
    /// Returns the total cohesion eroded across the body's cells during this step.
    pub fn sync_cohesion_from_lattice(
        &mut self,
        material: &MaterialField,
        state: &AcousticState,
        dt: f32,
        periodic: bool,
    ) -> f32 {
        let w = state.config.width as isize;
        let h = state.config.height as isize;
        let has_mat_coh = material.cohesion.len() == (w as usize) * (h as usize);
        let mut total_eroded = 0.0f32;

        for ly in 0..self.local_size {
            for lx in 0..self.local_size {
                let l_idx = ly * self.local_size + lx;
                if !self.occupied[l_idx] || self.cohesion[l_idx] <= 0.0 {
                    continue;
                }
                let (wx, wy) = self.local_to_world(lx as f32, ly as f32);
                let px = wx.round() as isize;
                let py = wy.round() as isize;
                if !periodic && (px < 1 || px >= w - 1 || py < 1 || py >= h - 1) {
                    continue;
                }
                let gx = px.rem_euclid(w) as usize;
                let gy = py.rem_euclid(h) as usize;
                let g_idx = state.config.index(gx, gy);

                let p_abs = state.pressure[g_idx].abs();
                if p_abs > self.material.yield_pressure {
                    let excess = p_abs - self.material.yield_pressure;
                    let delta = (0.22 * excess * dt).min(self.cohesion[l_idx]);
                    self.cohesion[l_idx] -= delta;
                    total_eroded += delta;
                } else if has_mat_coh {
                    let lat_c = material.cohesion[g_idx];
                    if lat_c > 0.0 && lat_c + 0.02 < self.cohesion[l_idx] {
                        let delta = (self.cohesion[l_idx] - lat_c) * 0.35;
                        self.cohesion[l_idx] = (self.cohesion[l_idx] - delta).max(0.0);
                        total_eroded += delta;
                    }
                }
            }
        }

        if total_eroded > 0.01 {
            self.recompute_mass_properties();
        }
        total_eroded
    }

    /// Advance body center-of-mass `(x, y)` and orientation `theta` by `dt_sub`, applying either
    /// toroidal periodic wrapping or bounded arena reflection with `wall_margin`.
    pub fn step_kinematics(
        &mut self,
        dt_sub: f32,
        width: f32,
        height: f32,
        periodic: bool,
        wall_margin: f32,
    ) {
        if periodic {
            self.x = (self.x + self.vx * dt_sub).rem_euclid(width);
            self.y = (self.y + self.vy * dt_sub).rem_euclid(height);
        } else {
            let mut x = self.x + self.vx * dt_sub;
            let mut y = self.y + self.vy * dt_sub;
            let min_x = self.radius + wall_margin;
            let max_x = width - self.radius - wall_margin;
            let min_y = self.radius + wall_margin;
            let max_y = height - self.radius - wall_margin;

            if x < min_x {
                x = min_x;
                self.vx = self.vx.abs();
            } else if x > max_x {
                x = max_x;
                self.vx = -self.vx.abs();
            }
            if y < min_y {
                y = min_y;
                self.vy = self.vy.abs();
            } else if y > max_y {
                y = max_y;
                self.vy = -self.vy.abs();
            }
            self.x = x;
            self.y = y;
        }
        self.theta = (self.theta + self.omega * dt_sub).rem_euclid(TAU);
    }
}

/// Resolve field-coupled elastic and progressive-fracture contact between two `CohesiveBody` hulls.
///
/// - Separates overlapping contours so solid bodies never interpenetrate.
/// - Applies equal-and-opposite off-center contact impulses (`apply_contact_impulse`) conserving
///   momentum and imparting spin (`r x J`).
/// - Radiates an acoustic impact shockwave (`GaussianImpulse`) into `state`.
/// - If `fracture_closing_speed` is `Some(thresh)` and the closing speed exceeds `thresh`,
///   carves progressive fractures into both colliding hulls.
/// - Returns `Some((contact_x, contact_y, closing_speed))` if a contact occurred.
pub fn resolve_body_pair_contact(
    body_a: &mut CohesiveBody,
    body_b: &mut CohesiveBody,
    state: &mut AcousticState,
    periodic_dims: Option<(f32, f32)>,
    overlap_scale: f32,
    fracture_closing_speed: Option<f32>,
    wave_force_enabled: bool,
) -> Option<(f32, f32, f32)> {
    let mut dx = body_b.x - body_a.x;
    let mut dy = body_b.y - body_a.y;
    if let Some((w, h)) = periodic_dims {
        if dx > 0.5 * w {
            dx -= w;
        } else if dx < -0.5 * w {
            dx += w;
        }
        if dy > 0.5 * h {
            dy -= h;
        } else if dy < -0.5 * h {
            dy += h;
        }
    }

    let min_dist = (body_a.radius + body_b.radius) * overlap_scale;
    let d2 = dx * dx + dy * dy;
    if d2 >= min_dist * min_dist || d2 <= 0.001 {
        return None;
    }

    let d = d2.sqrt();
    let nx = dx / d;
    let ny = dy / d;
    let overlap = min_dist - d;

    body_a.x -= nx * overlap * 0.52;
    body_a.y -= ny * overlap * 0.52;
    body_b.x += nx * overlap * 0.52;
    body_b.y += ny * overlap * 0.52;

    if let Some((w, h)) = periodic_dims {
        body_a.x = body_a.x.rem_euclid(w);
        body_a.y = body_a.y.rem_euclid(h);
        body_b.x = body_b.x.rem_euclid(w);
        body_b.y = body_b.y.rem_euclid(h);
    }

    let rvx = body_b.vx - body_a.vx;
    let rvy = body_b.vy - body_a.vy;
    let closing = -(rvx * nx + rvy * ny);
    let cx = body_a.x + nx * body_a.radius * 0.85;
    let cy = body_a.y + ny * body_a.radius * 0.85;

    if let Some(fracture_thresh) = fracture_closing_speed {
        let bump = (closing.max(0.12) * 1.65).min(2.4);
        body_a.apply_contact_impulse(cx, cy, -nx * bump, -ny * bump, periodic_dims);
        body_b.apply_contact_impulse(cx, cy, nx * bump, ny * bump, periodic_dims);

        if closing > fracture_thresh {
            body_a.inflict_progressive_fracture(cx, cy, -nx, -ny, 1.15, periodic_dims);
            body_b.inflict_progressive_fracture(cx, cy, nx, ny, 1.15, periodic_dims);
        }
    } else if closing > 0.0 {
        let imp = 1.5 * closing;
        body_a.vx -= nx * imp * 0.5;
        body_a.vy -= ny * imp * 0.5;
        body_b.vx += nx * imp * 0.5;
        body_b.vy += ny * imp * 0.5;
    }

    if wave_force_enabled {
        let w = state.config.width as f32;
        let h = state.config.height as f32;
        if periodic_dims.is_some() || (cx >= 2.0 && cx < w - 2.0 && cy >= 2.0 && cy < h - 2.0) {
            let amp = (closing.max(0.2) * 1.8).clamp(0.65, 2.2);
            GaussianImpulse::new(cx, cy, 1.5, amp).inject_wrapped(state, periodic_dims.is_some());
        }
    }

    Some((cx, cy, closing))
}
