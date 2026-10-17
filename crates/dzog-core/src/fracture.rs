//! Crystalline Cleavage Fissures, Multi-Stage Progressive Fracture, and Topological BFS Splitting.
//!
//! Separated from `body.rs` so hull generation/rasterization and fracture mechanics remain modular.

use std::collections::VecDeque;
use crate::body::CohesiveBody;

impl CohesiveBody {
    /// Carve directional crystalline cleavage planes through the body's local grid from world impact
    /// point `(wx, wy)` along propagation vector `(dir_x, dir_y)`.
    ///
    /// Returns `true` if this strike triggered a **Resonance Crit** (striking an already-cracked vein
    /// or using an overcharged bolt (`energy >= 1.8`) to cleave all the way through the crystal).
    pub fn carve_cleavage_fissure(
        &mut self,
        wx: f32,
        wy: f32,
        dir_x: f32,
        dir_y: f32,
        energy: f32,
        periodic_dims: Option<(f32, f32)>,
    ) -> bool {
        let (lx_raw, ly_raw) = self.world_to_local(wx, wy, periodic_dims);
        let n_f = (self.local_size.saturating_sub(1)) as f32;
        let lx = lx_raw.clamp(0.0, n_f);
        let ly = ly_raw.clamp(0.0, n_f);

        // Rotate propagation vector into body-local frame (-theta)
        let cos_t = self.theta.cos();
        let sin_t = self.theta.sin();
        let l_dx = dir_x * cos_t + dir_y * sin_t;
        let l_dy = -dir_x * sin_t + dir_y * cos_t;
        let base_angle = l_dy.atan2(l_dx);

        // Check if impact point is near an existing cracked cleavage vein (Resonance Crit!)
        let mut hit_existing_crack = false;
        for ny in 0..self.local_size {
            for nx in 0..self.local_size {
                let idx = ny * self.local_size + nx;
                if self.occupied[idx] && self.cohesion[idx] < 0.65 {
                    let d2 = (nx as f32 - lx).powi(2) + (ny as f32 - ly).powi(2);
                    if d2 <= 6.25 {
                        hit_existing_crack = true;
                        break;
                    }
                }
            }
        }

        let full_cleave = hit_existing_crack || energy >= 1.75 || self.mean_cohesion() < 0.78;

        // 1. Carve impact spall crater around (lx, ly)
        let crater_r = if full_cleave { 2.4 } else { 1.8 };
        for ny in 0..self.local_size {
            for nx in 0..self.local_size {
                let idx = ny * self.local_size + nx;
                if !self.occupied[idx] {
                    continue;
                }
                let dist = ((nx as f32 - lx).powi(2) + (ny as f32 - ly).powi(2)).sqrt();
                if dist <= crater_r {
                    let target_c = if full_cleave { 0.0 } else { 0.25 };
                    self.cohesion[idx] = self.cohesion[idx].min(target_c);
                }
            }
        }

        // 2. Trace primary cleavage ray + two shear branches
        let branches: &[(f32, f32)] = if full_cleave {
            // Through-going cleavage plane that severs the crystal into 2-3 topological components
            &[(0.0, 0.0), (std::f32::consts::PI, 0.0), (0.62, 0.08)]
        } else {
            // Partial waveguide fissure (0.28 cohesion) that channels phonons without severing yet
            &[(0.0, 0.28), (0.50, 0.35), (-0.50, 0.35)]
        };

        let max_steps = self.local_size * 2;
        for &(ang_off, target_coh) in branches {
            let ang = base_angle + ang_off;
            let vx = ang.cos();
            let vy = ang.sin();
            let perp_x = -vy;
            let perp_y = vx;

            for step in 0..max_steps {
                let t = step as f32 * 0.45;
                let px = lx + vx * t;
                let py = ly + vy * t;

                // Ensure 4-connected separation by carving a 1.5-cell wide seam when full_cleave is active
                let widths: &[f32] = if full_cleave { &[-0.55, 0.0, 0.55] } else { &[0.0] };
                for &w_off in widths {
                    let ix = (px + perp_x * w_off).round() as isize;
                    let iy = (py + perp_y * w_off).round() as isize;
                    if ix >= 0 && ix < self.local_size as isize && iy >= 0 && iy < self.local_size as isize {
                        let idx = (iy as usize) * self.local_size + (ix as usize);
                        if self.occupied[idx] {
                            self.cohesion[idx] = self.cohesion[idx].min(target_coh);
                        }
                    }
                }
            }
        }

        full_cleave
    }

    /// Progressive multi-stage fracture propagation for medium and large hulls & structures.
    ///
    /// - Initial strikes on intact armor carve localized glowing fissures (`C in 0.22..0.38`) without
    ///   instantly severing the hull.
    /// - Subsequent strikes hitting near existing fractures (`C < 0.68`) amplify fracture growth:
    ///   eroding existing cracks toward cavitation (`C -> 0.0`), extending fracture length, and
    ///   branching secondary cleavage seams until the section topologically cleaves via BFS.
    /// - Returns `(triggered_full_cleave, existing_fractured_cells_hit)`.
    pub fn inflict_progressive_fracture(
        &mut self,
        wx: f32,
        wy: f32,
        dir_x: f32,
        dir_y: f32,
        energy: f32,
        periodic_dims: Option<(f32, f32)>,
    ) -> (bool, usize) {
        let (lx_raw, ly_raw) = self.world_to_local(wx, wy, periodic_dims);
        let n_f = (self.local_size.saturating_sub(1)) as f32;
        let lx = lx_raw.clamp(0.0, n_f);
        let ly = ly_raw.clamp(0.0, n_f);

        let cos_t = self.theta.cos();
        let sin_t = self.theta.sin();
        let l_dx = dir_x * cos_t + dir_y * sin_t;
        let l_dy = -dir_x * sin_t + dir_y * cos_t;
        let base_angle = l_dy.atan2(l_dx);

        // Count existing fractured cells near the impact site and sympathetically deepen them
        let mut nearby_cracked = 0usize;
        for ny in 0..self.local_size {
            for nx in 0..self.local_size {
                let idx = ny * self.local_size + nx;
                if !self.occupied[idx] {
                    continue;
                }
                let d2 = (nx as f32 - lx).powi(2) + (ny as f32 - ly).powi(2);
                if d2 <= 12.25 && self.cohesion[idx] < 0.68 {
                    nearby_cracked += 1;
                    // Hitting an existing fracture erodes surrounding cracked cells further
                    let erode = (0.20 * energy * (1.0 - (d2.sqrt() / 3.8).min(0.85))).max(0.06);
                    self.cohesion[idx] = (self.cohesion[idx] - erode).max(0.0);
                }
            }
        }

        let mean_c = self.mean_cohesion();
        let stage = if energy >= 2.05 || nearby_cracked >= 4 || mean_c < 0.72 {
            3 // Full through-cleave stage
        } else if nearby_cracked >= 1 || mean_c < 0.86 {
            2 // Branched deep fracture stage
        } else {
            1 // Initial surface & waveguide crack stage
        };

        let crater_r = match stage {
            1 => 0.85,
            2 => 1.15,
            _ => 1.45,
        };
        let crater_coh = match stage {
            1 => 0.30,
            2 => 0.10,
            _ => 0.0,
        };
        for ny in 0..self.local_size {
            for nx in 0..self.local_size {
                let idx = ny * self.local_size + nx;
                if !self.occupied[idx] {
                    continue;
                }
                let dist = ((nx as f32 - lx).powi(2) + (ny as f32 - ly).powi(2)).sqrt();
                if dist <= crater_r {
                    self.cohesion[idx] = self.cohesion[idx].min(crater_coh);
                }
            }
        }

        let branches: &[(f32, f32, f32)] = match stage {
            1 => &[
                (0.0, 0.32, 0.52),
                (0.48, 0.38, 0.36),
                (-0.48, 0.38, 0.36),
            ],
            2 => &[
                (0.0, 0.08, 0.85),
                (std::f32::consts::PI, 0.08, 0.65),
                (0.55, 0.22, 0.60),
                (-0.55, 0.22, 0.60),
            ],
            _ => &[
                (0.0, 0.0, 1.40),
                (std::f32::consts::PI, 0.0, 1.40),
                (0.58, 0.0, 1.10),
                (-0.58, 0.08, 0.90),
            ],
        };

        let full_cleave = stage >= 3;
        for &(ang_off, target_coh, len_frac) in branches {
            let ang = base_angle + ang_off;
            let vx = ang.cos();
            let vy = ang.sin();
            let perp_x = -vy;
            let perp_y = vx;
            let max_steps = ((self.local_size as f32 * len_frac * 1.6) as usize).max(4);
            let widths: &[f32] = if full_cleave && target_coh == 0.0 {
                &[-0.55, 0.0, 0.55]
            } else if stage >= 2 {
                &[-0.35, 0.0, 0.35]
            } else {
                &[0.0]
            };

            for step in 0..max_steps {
                let t = step as f32 * 0.45;
                let px = lx + vx * t;
                let py = ly + vy * t;
                for &w_off in widths {
                    let ix = (px + perp_x * w_off).round() as isize;
                    let iy = (py + perp_y * w_off).round() as isize;
                    if ix >= 0
                        && ix < self.local_size as isize
                        && iy >= 0
                        && iy < self.local_size as isize
                    {
                        let idx = (iy as usize) * self.local_size + (ix as usize);
                        if self.occupied[idx] {
                            self.cohesion[idx] = self.cohesion[idx].min(target_coh);
                        }
                    }
                }
            }
        }

        self.recompute_mass_properties();
        (full_cleave, nearby_cracked)
    }

    /// Run 4-connected Breadth-First Search (BFS) over intact/waveguide cells (`C >= 0.15`).
    ///
    /// If the body's topology has been cleaved into `k >= 2` connected components (each >= 4 cells),
    /// splits the parent body into `k` independent child `CohesiveBody` fragments conserving
    /// linear and angular momentum plus outward cavitation separation velocity.
    pub fn split_connected_components(
        &self,
        next_id: &mut u32,
        periodic_dims: Option<(f32, f32)>,
    ) -> Vec<CohesiveBody> {
        let n = self.local_size;
        let count = n * n;
        let mut visited = vec![false; count];
        let mut components: Vec<Vec<(usize, usize, f32)>> = Vec::new();

        for sy in 0..n {
            for sx in 0..n {
                let s_idx = sy * n + sx;
                if visited[s_idx] || !self.occupied[s_idx] || self.cohesion[s_idx] < 0.15 {
                    continue;
                }

                let mut comp = Vec::new();
                let mut queue = VecDeque::new();
                visited[s_idx] = true;
                queue.push_back((sx, sy));

                while let Some((cx, cy)) = queue.pop_front() {
                    let c_idx = cy * n + cx;
                    comp.push((cx, cy, self.cohesion[c_idx]));

                    let neighbors = [
                        (cx.wrapping_sub(1), cy),
                        (cx + 1, cy),
                        (cx, cy.wrapping_sub(1)),
                        (cx, cy + 1),
                    ];
                    for (nx, ny) in neighbors {
                        if nx < n && ny < n {
                            let n_idx = ny * n + nx;
                            if !visited[n_idx] && self.occupied[n_idx] && self.cohesion[n_idx] >= 0.15 {
                                visited[n_idx] = true;
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                }

                if comp.len() >= 4 {
                    components.push(comp);
                }
            }
        }

        if components.len() < 2 {
            return Vec::new();
        }

        let mut children = Vec::with_capacity(components.len());

        for comp in components {
            // Compute component centroid in parent local coordinates
            let mut sum_lx = 0.0f32;
            let mut sum_ly = 0.0f32;
            for &(lx, ly, _) in &comp {
                sum_lx += lx as f32;
                sum_ly += ly as f32;
            }
            let mean_lx = sum_lx / comp.len() as f32;
            let mean_ly = sum_ly / comp.len() as f32;

            // Map child centroid to world coordinates
            let (mut world_cx, mut world_cy) = self.local_to_world(mean_lx, mean_ly);
            if let Some((w, h)) = periodic_dims {
                world_cx = world_cx.rem_euclid(w);
                world_cy = world_cy.rem_euclid(h);
            }

            // Determine bounding radius of child component
            let mut max_r = 2.0f32;
            for &(lx, ly, _) in &comp {
                let d = ((lx as f32 - mean_lx).powi(2) + (ly as f32 - mean_ly).powi(2)).sqrt();
                if d > max_r {
                    max_r = d;
                }
            }
            let child_size = ((max_r * 2.4).ceil() as usize).clamp(6, 20);
            let child_center = (child_size as f32 - 1.0) * 0.5;
            let mut child_occ = vec![false; child_size * child_size];
            let mut child_coh = vec![0.0f32; child_size * child_size];

            for &(lx, ly, coh) in &comp {
                let clx = (lx as f32 - mean_lx + child_center).round() as isize;
                let cly = (ly as f32 - mean_ly + child_center).round() as isize;
                if clx >= 0 && clx < child_size as isize && cly >= 0 && cly < child_size as isize {
                    let c_idx = (cly as usize) * child_size + (clx as usize);
                    child_occ[c_idx] = true;
                    // Freshly cleaved crystal faces anneal to solid cohesion (0.88)
                    child_coh[c_idx] = coh.max(0.88);
                }
            }

            // Momentum partition: v_child = v_cm + omega x r_offset + cavitation separation impulse
            let (raw_wx, raw_wy) = self.local_to_world(mean_lx, mean_ly);
            let rx = raw_wx - self.x;
            let ry = raw_wy - self.y;
            let r_len = (rx * rx + ry * ry).sqrt().max(0.5);
            let sep_speed = 0.14;
            let child_vx = (self.vx - self.omega * ry + (rx / r_len) * sep_speed).clamp(-0.60, 0.60);
            let child_vy = (self.vy + self.omega * rx + (ry / r_len) * sep_speed).clamp(-0.60, 0.60);
            let child_omega = (self.omega + (rx * sep_speed - ry * sep_speed) * 0.04).clamp(-0.14, 0.14);

            let child_id = *next_id;
            *next_id += 1;

            let mut child = CohesiveBody {
                id: child_id,
                x: world_cx,
                y: world_cy,
                vx: child_vx,
                vy: child_vy,
                theta: self.theta,
                omega: child_omega,
                local_size: child_size,
                occupied: child_occ,
                cohesion: child_coh,
                material: self.material,
                mass: 1.0,
                inertia: 1.0,
                radius: max_r,
                last_fracture_step: self.last_fracture_step,
            };
            child.recompute_mass_properties();
            children.push(child);
        }

        children
    }
}
