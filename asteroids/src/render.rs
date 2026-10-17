//! Perceptual Field & Crystalline Actor Renderer for `asteroids` (`asteroids/src/render.rs`).

use std::f32::consts::TAU;
use dzog_bevy::FieldViewMode;
use dzog_core::MineralClass;
use super::{AlienKind, AsteroidsFieldGame};

impl AsteroidsFieldGame {
    /// Render the current field & actor ecology into `self.pixel_buffer` (`RGBA8`).
    pub fn render_to_pixel_buffer(&mut self) {
        let w = self.width;
        let h = self.height;
        let periodic = self.is_periodic();

        if self.pixel_buffer.len() != w * h * 4 {
            self.pixel_buffer.resize(w * h * 4, 0);
        }

        let has_lum = self.solver.state.luminescence.len() == w * h;

        // 1. Background field & Radiative Luminescence L(x, y) projection
        for y in 0..h {
            for x in 0..w {
                let idx = self.solver.state.config.index(x, y);
                let p = self.solver.state.pressure[idx];
                let vx = self.solver.state.vx[idx];
                let vy = self.solver.state.vy[idx];
                let lum = if has_lum { self.solver.state.luminescence[idx] } else { 0.0 };
                let rho = self.solver.material.density[idx];
                let k = self.solver.material.bulk_modulus[idx];
                let coh = self.solver.material.cohesion[idx];

                // Deterministic deep-space starfield shimmer in quiet vacuum
                let star_hash = (x as u32)
                    .wrapping_mul(374761393)
                    .wrapping_add((y as u32).wrapping_mul(668265263));
                let star = if (star_hash & 0x1FF) == 0 {
                    0.18 + ((star_hash >> 9) & 0x7) as f32 * 0.03
                } else {
                    0.0
                };

                let (mut r, mut g, mut b): (f32, f32, f32) = match self.view_mode {
                    FieldViewMode::CompositeArcade => {
                        let p_pos = p.max(0.0);
                        let p_neg = (-p).max(0.0);
                        let v_mag = (vx * vx + vy * vy).sqrt();
                        // Field-driven exciton luminescence glow (plasma cyan / solar gold)
                        let l_glow = lum.min(3.0);
                        (
                            0.015 + star * 0.8 + p_pos * 0.42 + v_mag * 0.14 + l_glow * 0.24,
                            0.022 + star * 0.9 + p_pos * 0.26 + p_neg * 0.34 + v_mag * 0.24 + l_glow * 0.62,
                            0.048 + star + p_neg * 0.62 + p_pos * 0.22 + l_glow * 0.82,
                        )
                    }
                    FieldViewMode::PressureAndBohmian => {
                        let q = self
                            .solver
                            .state
                            .quantum_potential_wrapped_at(x, y, periodic);
                        let p_pos = p.max(0.0);
                        let p_neg = (-p).max(0.0);
                        let q_well = (-q).max(0.0) * 0.08;
                        (
                            0.03 + p_pos * 0.65 + q_well * 0.55 + lum * 0.20,
                            0.03 + q_well * 0.85 + lum * 0.45,
                            0.06 + p_neg * 0.70 + q_well * 0.40 + lum * 0.55,
                        )
                    }
                    FieldViewMode::VelocityAndFlux => {
                        let v_mag = (vx * vx + vy * vy).sqrt();
                        let flux = (p * vx).hypot(p * vy);
                        let phi_g = (-self.gravity.potential[idx]).max(0.0);
                        (
                            0.03 + flux * 0.65 + phi_g * 0.35 + lum * 0.30,
                            0.04 + v_mag * 0.75 + lum * 0.50,
                            0.07 + v_mag * 0.45 + phi_g * 0.75 + lum * 0.40,
                        )
                    }
                    FieldViewMode::ImpedanceAndCohesion => {
                        let z = (rho * k).sqrt();
                        let z_dev = (z - 1.0).abs();
                        (
                            0.04 + z_dev * 0.70,
                            0.04 + coh * 0.65,
                            0.07 + (1.0 - coh) * z_dev * 0.85,
                        )
                    }
                };

                if self.view_mode == FieldViewMode::ImpedanceAndCohesion && coh >= 0.15 {
                    r += 0.25 * (1.0 - coh);
                    g += 0.35 * coh;
                    b += 0.30;
                }

                let p_idx = (y * w + x) * 4;
                self.pixel_buffer[p_idx] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                self.pixel_buffer[p_idx + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                self.pixel_buffer[p_idx + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
                self.pixel_buffer[p_idx + 3] = 255;
            }
        }

        // 2. Render Faceted 3D Mineralogical Asteroids with Dynamic Field Illumination
        for rock in &self.asteroids {
            let r_bound = (rock.radius.ceil() as isize) + 1;
            let cx = rock.x.round() as isize;
            let cy = rock.y.round() as isize;
            let cos_t = rock.theta.cos();
            let sin_t = rock.theta.sin();
            let center = (rock.local_size as f32 - 1.0) * 0.5;

            for oy in -r_bound..=r_bound {
                for ox in -r_bound..=r_bound {
                    let px = cx + ox;
                    let py = cy + oy;
                    if !periodic && (px < 0 || px >= w as isize || py < 0 || py >= h as isize) {
                        continue;
                    }
                    let gx = px.rem_euclid(w as isize) as usize;
                    let gy = py.rem_euclid(h as isize) as usize;

                    let dx = px as f32 - rock.x;
                    let dy = py as f32 - rock.y;
                    let lx = dx * cos_t + dy * sin_t + center;
                    let ly = -dx * sin_t + dy * cos_t + center;
                    let coh = rock.sample_local_cohesion(lx, ly);
                    if coh < 0.12 {
                        continue;
                    }

                    let idx = self.solver.state.config.index(gx, gy);
                    let p_val = self.solver.state.pressure[idx];
                    let lum_val = if has_lum { self.solver.state.luminescence[idx] } else { 0.0 };
                    let (glx, gly) = self
                        .solver
                        .state
                        .luminescence_gradient_wrapped_at(gx, gy, periodic);
                    let (gpx, gpy) = self
                        .solver
                        .state
                        .pressure_gradient_wrapped_at(gx, gy, periodic);

                    let (elev, w_nx, w_ny, ridge) = rock.sample_crystal_facet(lx, ly);

                    // Zenith starlight (-0.45, -0.65) + dynamic field illumination from nearby solitons/blasts
                    let zenith_dot = (-0.45 * w_nx - 0.65 * w_ny + 0.60 * elev).clamp(0.18, 1.25);
                    let field_light_x = glx * 1.4 + gpx.abs() * 0.35;
                    let field_light_y = gly * 1.4 + gpy.abs() * 0.35;
                    let field_dot = (w_nx * field_light_x + w_ny * field_light_y).max(0.0)
                        + lum_val * 0.42
                        + p_val.abs().min(2.0) * 0.22;

                    let (r, g, b) = if coh < 0.55 {
                        // Glowing semi-permeable internal cleavage waveguide vein!
                        let vein_pulse = (p_val.abs() + lum_val).min(2.5) * 0.35;
                        match rock.material.class {
                            MineralClass::PiezoQuartz => (
                                0.55 + vein_pulse * 0.45,
                                0.96,
                                1.0,
                            ),
                            MineralClass::ObsidianAnvil => (
                                1.0,
                                0.52 + vein_pulse * 0.40,
                                0.18 + vein_pulse * 0.25,
                            ),
                            _ => (
                                0.98,
                                0.88,
                                0.38 + vein_pulse * 0.45,
                            ),
                        }
                    } else {
                        match rock.material.class {
                            MineralClass::PiezoQuartz => {
                                // Translucent cyan-amethyst hexagonal crystal with bright prism ridges
                                let base_r = 0.18 * zenith_dot + ridge * 0.28 + field_dot * 0.38;
                                let base_g = 0.62 * zenith_dot + ridge * 0.34 + field_dot * 0.52;
                                let base_b = 0.88 * zenith_dot + ridge * 0.26 + field_dot * 0.45;
                                (base_r, base_g, base_b)
                            }
                            MineralClass::ObsidianAnvil => {
                                // Dark volcanic basalt-obsidian with razor-sharp crimson-silver specular bevels
                                let spec = (ridge * 0.52 + field_dot * 0.55).min(0.95);
                                let base_r = 0.24 * zenith_dot + spec * 0.72;
                                let base_g = 0.14 * zenith_dot + spec * 0.30;
                                let base_b = 0.22 * zenith_dot + spec * 0.44;
                                (base_r, base_g, base_b)
                            }
                            MineralClass::BismuthCrystal => {
                                // Stepped hopper-crystal terraces with thin-film amber/teal/magenta iridescence
                                let irid_phase = (lx * 0.55 + ly * 0.45 + p_val * 0.8).sin() * 0.5 + 0.5;
                                let base_r = (0.68 - 0.28 * irid_phase) * zenith_dot + ridge * 0.32 + field_dot * 0.40;
                                let base_g = (0.42 + 0.32 * irid_phase) * zenith_dot + ridge * 0.28 + field_dot * 0.42;
                                let base_b = (0.24 + 0.48 * (1.0 - irid_phase)) * zenith_dot + ridge * 0.35 + field_dot * 0.45;
                                (base_r, base_g, base_b)
                            }
                            MineralClass::AlienResonator => (
                                0.20 * zenith_dot + ridge * 0.3,
                                0.82 * zenith_dot + ridge * 0.2,
                                0.52 * zenith_dot + ridge * 0.3,
                            ),
                        }
                    };

                    // Smooth edge blending at crystal silhouette boundary
                    let alpha = ((coh - 0.12) / 0.30).clamp(0.25, 1.0);
                    let p_idx = (gy * w + gx) * 4;
                    let bg_r = self.pixel_buffer[p_idx] as f32 / 255.0;
                    let bg_g = self.pixel_buffer[p_idx + 1] as f32 / 255.0;
                    let bg_b = self.pixel_buffer[p_idx + 2] as f32 / 255.0;

                    let out_r = bg_r * (1.0 - alpha) + r.clamp(0.0, 1.0) * alpha;
                    let out_g = bg_g * (1.0 - alpha) + g.clamp(0.0, 1.0) * alpha;
                    let out_b = bg_b * (1.0 - alpha) + b.clamp(0.0, 1.0) * alpha;

                    self.pixel_buffer[p_idx] = (out_r.clamp(0.0, 1.0) * 255.0) as u8;
                    self.pixel_buffer[p_idx + 1] = (out_g.clamp(0.0, 1.0) * 255.0) as u8;
                    self.pixel_buffer[p_idx + 2] = (out_b.clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }

        // 3. Render Ship-Like Alien Enemy Craft (Bohmian Saucer Corvette & Harmonic Weaver Cruiser)
        for alien in &self.aliens {
            let ax = alien.hull.x;
            let ay = alien.hull.y;
            let r_bound = (alien.hull.radius.ceil() as isize) + 2;

            match alien.kind {
                AlienKind::BohmianSaucer => {
                    // Draw telegraphed lock-on laser sightline before firing
                    if alien.telegraph_timer > 0 {
                        let aim_cos = alien.aim_angle.cos();
                        let aim_sin = alien.aim_angle.sin();
                        for step in 4..28 {
                            let d = step as f32 * 1.1;
                            let gx = ((ax + aim_cos * d).round() as isize).rem_euclid(w as isize) as usize;
                            let gy = ((ay + aim_sin * d).round() as isize).rem_euclid(h as isize) as usize;
                            let p_idx = (gy * w + gx) * 4;
                            self.pixel_buffer[p_idx] = 255;
                            self.pixel_buffer[p_idx + 1] = 85;
                            self.pixel_buffer[p_idx + 2] = 120;
                        }
                    }

                    // Articulated Saucer Corvette: swept wing sponsons, rotating phase emitter nodes, glowing bridge dome
                    let cos_t = alien.hull.theta.cos();
                    let sin_t = alien.hull.theta.sin();
                    let cx_i = ax.round() as isize;
                    let cy_i = ay.round() as isize;

                    for oy in -r_bound..=r_bound {
                        for ox in -r_bound..=r_bound {
                            let wx = cx_i + ox;
                            let wy = cy_i + oy;
                            let dx = wx as f32 - ax;
                            let dy = wy as f32 - ay;
                            // Saucer-local coordinates
                            let lx = dx * cos_t + dy * sin_t;
                            let ly = -dx * sin_t + dy * cos_t;

                            let disc_r = (lx * lx + (ly * 1.32).powi(2)).sqrt();
                            let wing_sponson = lx.abs() <= alien.hull.radius * 1.12
                                && ly.abs() <= 1.45
                                && lx.abs() >= alien.hull.radius * 0.55;
                            if disc_r > alien.hull.radius && !wing_sponson {
                                continue;
                            }

                            let gx = wx.rem_euclid(w as isize) as usize;
                            let gy = wy.rem_euclid(h as isize) as usize;
                            let p_idx = (gy * w + gx) * 4;

                            let col: [u8; 4] = if disc_r < 1.85 {
                                // Bioluminescent cyan-white command bridge canopy
                                [165, 255, 245, 255]
                            } else if (disc_r - 3.3).abs() < 0.65 {
                                // Rotating golden-crimson phase emitter ring
                                let ang = ly.atan2(lx);
                                let node = ((ang * 3.0).cos() > 0.35) as u8;
                                if node == 1 {
                                    [255, 215, 95, 255]
                                } else {
                                    [35, 135, 105, 255]
                                }
                            } else if wing_sponson {
                                // Swept wingtip weapon pods
                                [255, 95, 125, 255]
                            } else {
                                // Armored emerald-slate corvette hull plating
                                [48, 185, 130, 255]
                            };
                            self.pixel_buffer[p_idx..p_idx + 4].copy_from_slice(&col);
                        }
                    }
                }
                AlienKind::HarmonicWeaver => {
                    // Twin-Prong Acoustic Cruiser oriented along `alien.aim_angle`
                    let cos_a = alien.aim_angle.cos();
                    let sin_a = alien.aim_angle.sin();
                    let cx_i = ax.round() as isize;
                    let cy_i = ay.round() as isize;

                    // Active standing-wave resonance beam between prongs and target rock
                    if alien.telegraph_timer > 0 || alien.fire_cooldown > 34 {
                        for step in 3..16 {
                            let d = step as f32 * 0.9;
                            let wave_w = (step as f32 * 1.1).sin() * 1.2;
                            let wx = ax + cos_a * d - sin_a * wave_w;
                            let wy = ay + sin_a * d + cos_a * wave_w;
                            let gx = (wx.round() as isize).rem_euclid(w as isize) as usize;
                            let gy = (wy.round() as isize).rem_euclid(h as isize) as usize;
                            let p_idx = (gy * w + gx) * 4;
                            self.pixel_buffer[p_idx..p_idx + 4]
                                .copy_from_slice(&[255, 195, 90, 255]);
                        }
                    }

                    for oy in -r_bound..=r_bound {
                        for ox in -r_bound..=r_bound {
                            let wx = cx_i + ox;
                            let wy = cy_i + oy;
                            let dx = wx as f32 - ax;
                            let dy = wy as f32 - ay;
                            // Ship-local frame (+lx = forward along aim_angle)
                            let lx = dx * cos_a + dy * sin_a;
                            let ly = -dx * sin_a + dy * cos_a;

                            let aft_fuselage = lx >= -4.2 && lx <= 1.2 && ly.abs() <= (3.4 - (lx + 1.5).abs() * 0.45);
                            let forward_prongs = lx > 0.5 && lx <= 5.2 && (ly.abs() >= 1.25 && ly.abs() <= 2.65);
                            let engine_pod = lx < -3.2 && lx >= -5.0 && ly.abs() <= 1.6;

                            if !aft_fuselage && !forward_prongs && !engine_pod {
                                continue;
                            }

                            let gx = wx.rem_euclid(w as isize) as usize;
                            let gy = wy.rem_euclid(h as isize) as usize;
                            let p_idx = (gy * w + gx) * 4;

                            let col: [u8; 4] = if engine_pod {
                                // Glowing aft plasma thruster
                                [255, 120, 60, 255]
                            } else if lx.hypot(ly) <= 1.65 {
                                // Harmonic reactor core
                                [255, 235, 130, 255]
                            } else if forward_prongs && lx > 3.4 {
                                // Tuning-fork resonance emitter tips
                                [140, 255, 235, 255]
                            } else if forward_prongs {
                                // Twin forward resonance mandibles
                                [225, 155, 70, 255]
                            } else {
                                // Cruiser hull armor
                                [75, 175, 165, 255]
                            };
                            self.pixel_buffer[p_idx..p_idx + 4].copy_from_slice(&col);
                        }
                    }
                }
            }
        }

        // 4. Render Oriented Soliton Laser Bolts with Field-Coupled Glow Halos & Mach Shock Wings
        for bolt in &self.bolts {
            let speed = (bolt.vx * bolt.vx + bolt.vy * bolt.vy).sqrt().max(0.01);
            let dir_x = bolt.vx / speed;
            let dir_y = bolt.vy / speed;

            let (core_col, halo_col, tail_len, halo_r): ([f32; 3], [f32; 3], f32, isize) =
                if !bolt.from_player {
                    ([1.0, 0.88, 0.92], [1.0, 0.18, 0.38], 4.5, 4)
                } else if bolt.overcharged {
                    ([1.0, 1.0, 0.90], [1.0, 0.82, 0.25], 7.0, 5)
                } else {
                    ([0.92, 1.0, 1.0], [0.20, 0.88, 1.0], 5.2, 4)
                };

            let cx_i = bolt.x.round() as isize;
            let cy_i = bolt.y.round() as isize;

            for oy in -halo_r..=halo_r {
                for ox in -halo_r..=halo_r {
                    let wx = cx_i + ox;
                    let wy = cy_i + oy;
                    if !periodic && (wx < 0 || wx >= w as isize || wy < 0 || wy >= h as isize) {
                        continue;
                    }
                    let dx = wx as f32 - bolt.x;
                    let dy = wy as f32 - bolt.y;

                    // Project into bolt-aligned coordinates: `along` (+ = nose, - = wake tail), `perp`
                    let along = dx * dir_x + dy * dir_y;
                    let perp = -dx * dir_y + dy * dir_x;

                    let eff_along = if along < 0.0 {
                        along / (tail_len * 0.45)
                    } else {
                        along / 1.45
                    };
                    let d_ellip = (eff_along * eff_along + (perp / 1.35).powi(2)).sqrt();
                    if d_ellip > 2.6 {
                        continue;
                    }

                    let gx = wx.rem_euclid(w as isize) as usize;
                    let gy = wy.rem_euclid(h as isize) as usize;
                    let idx = self.solver.state.config.index(gx, gy);
                    let field_lum = if has_lum {
                        self.solver.state.luminescence[idx].min(2.5)
                    } else {
                        0.5
                    };

                    let core_w = (-d_ellip * d_ellip * 1.65).exp();
                    let halo_w = ((-d_ellip * d_ellip * 0.48).exp()) * (0.65 + 0.35 * field_lum);

                    let p_idx = (gy * w + gx) * 4;
                    let bg_r = self.pixel_buffer[p_idx] as f32 / 255.0;
                    let bg_g = self.pixel_buffer[p_idx + 1] as f32 / 255.0;
                    let bg_b = self.pixel_buffer[p_idx + 2] as f32 / 255.0;

                    let r = bg_r + halo_col[0] * halo_w * 0.78 + core_col[0] * core_w * 0.95;
                    let g = bg_g + halo_col[1] * halo_w * 0.78 + core_col[1] * core_w * 0.95;
                    let b = bg_b + halo_col[2] * halo_w * 0.78 + core_col[2] * core_w * 0.95;

                    self.pixel_buffer[p_idx] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                    self.pixel_buffer[p_idx + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                    self.pixel_buffer[p_idx + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }

        // 5. Overlay Player Wave-Interceptor, Thruster Plume & Antiphase Shield Ring
        let sx = self.ship_x;
        let sy = self.ship_y;
        let cos_t = self.ship_theta.cos();
        let sin_t = self.ship_theta.sin();

        // Render thruster plasma plume when thrusting
        if self.ship_thrusting_timer > 0 {
            for step in 3..=7 {
                let lx = -(step as f32);
                for ly_i in -1isize..=1 {
                    let ly = ly_i as f32 * (1.0 - (step - 3) as f32 * 0.18);
                    let wx = (sx + lx * cos_t - ly * sin_t).round() as isize;
                    let wy = (sy + lx * sin_t + ly * cos_t).round() as isize;
                    let gx = wx.rem_euclid(w as isize) as usize;
                    let gy = wy.rem_euclid(h as isize) as usize;
                    let p_idx = (gy * w + gx) * 4;
                    let plume_col = if step <= 4 {
                        [255, 240, 150, 255]
                    } else {
                        [90, 210, 255, 255]
                    };
                    self.pixel_buffer[p_idx..p_idx + 4].copy_from_slice(&plume_col);
                }
            }
        }

        for ly in -4isize..=4 {
            for lx in -3isize..=5 {
                let ly_abs = ly.abs() as f32;
                let in_hull = lx >= -2 && lx <= 4 && ly_abs <= (3.6 - lx as f32 * 0.62);
                if in_hull {
                    let wx = (sx + lx as f32 * cos_t - ly as f32 * sin_t).round() as isize;
                    let wy = (sy + lx as f32 * sin_t + ly as f32 * cos_t).round() as isize;
                    let gx = wx.rem_euclid(w as isize) as usize;
                    let gy = wy.rem_euclid(h as isize) as usize;
                    let p_idx = (gy * w + gx) * 4;
                    let ship_col = if lx >= 3 {
                        // Nose lance emitter
                        [255, 255, 255, 255]
                    } else if lx >= 0 && lx <= 1 && ly == 0 {
                        // Cockpit canopy
                        [80, 245, 255, 255]
                    } else if lx == -2 && ly.abs() >= 2 {
                        // Wingtip binaural acoustic probes
                        [255, 200, 90, 255]
                    } else if self.capacitor >= 35.0 {
                        [115, 230, 255, 255]
                    } else {
                        [175, 205, 235, 255]
                    };
                    self.pixel_buffer[p_idx..p_idx + 4].copy_from_slice(&ship_col);
                }
            }
        }

        if self.shield_visual_timer > 0 {
            let sr = self.ship_radius + 5.0;
            for step in 0..48 {
                let ang = (step as f32 / 48.0) * TAU;
                let wx = (sx + ang.cos() * sr).round() as isize;
                let wy = (sy + ang.sin() * sr).round() as isize;
                let gx = wx.rem_euclid(w as isize) as usize;
                let gy = wy.rem_euclid(h as isize) as usize;
                let p_idx = (gy * w + gx) * 4;
                self.pixel_buffer[p_idx..p_idx + 4].copy_from_slice(&[130, 255, 220, 255]);
            }
        }
    }
}
