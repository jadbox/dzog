use dzog_core::*;

#[test]
fn test_soliton_cohesion_vs_linear_dispersion() {
    let grid = GridConfig::new(80, 20, 1.0, 1.0);
    let material = MaterialField::homogeneous(grid, 1.0, 1.0, 0.0);
    let boundary = BoundaryMask::new(grid, OuterBoundaryRule::RigidWall);
    let dt = SolverConfig::compute_cfl_dt(&grid, &material, 0.7);

    // Case 1: Pure linear acoustics (beta = 0.0)
    let config_linear = SolverConfig::new(dt, 0.7, 1);
    let mut state_linear = AcousticState::new(grid);
    let mut packet1 = DirectionalWavePacket::new(15.0, 10.0, 2.5, 1.5, 0.0);
    packet1.inject(&mut state_linear, &material);
    let mut solver_linear = AcousticSolver::new(state_linear, material.clone(), boundary.clone(), config_linear).unwrap();

    // Case 2: Nonlinear solitary field (beta = 0.8, alpha = 0.5)
    let nl = NonlinearLaw::new(0.8, 0.5, 0.05);
    let config_soliton = SolverConfig::new(dt, 0.7, 1).with_nonlinearity(nl);
    let mut state_soliton = AcousticState::new(grid);
    let mut packet2 = DirectionalWavePacket::new(15.0, 10.0, 2.5, 1.5, 0.0);
    packet2.inject(&mut state_soliton, &material);
    let mut solver_soliton = AcousticSolver::new(state_soliton, material, boundary, config_soliton).unwrap();

    // Advance both solvers for 60 steps as the packets propagate towards x = 50
    for _ in 0..60 {
        solver_linear.step();
        solver_soliton.step();
    }

    // Measure peak amplitude of both packets
    let max_p_linear = solver_linear.state.pressure.iter().copied().fold(0.0f32, f32::max);
    let max_p_soliton = solver_soliton.state.pressure.iter().copied().fold(0.0f32, f32::max);

    println!("After 60 steps: Linear peak = {max_p_linear:.3}, Soliton peak = {max_p_soliton:.3}");

    // The nonlinear solitary self-focusing maintains peak concentration and counteracts 2D dispersion
    assert!(
        max_p_soliton > max_p_linear,
        "Soliton field must preserve cohesive peak amplitude higher than dispersive linear wave (soliton={max_p_soliton:.3} > linear={max_p_linear:.3})"
    );
}

#[test]
fn test_soliton_wall_reflection_rebound() {
    let grid = GridConfig::new(50, 16, 1.0, 1.0);
    let material = MaterialField::homogeneous(grid, 1.0, 1.0, 0.0);
    let boundary = BoundaryMask::new(grid, OuterBoundaryRule::RigidWall);
    let dt = SolverConfig::compute_cfl_dt(&grid, &material, 0.7);

    let nl = NonlinearLaw::new(0.6, 0.4, 0.02);
    let config = SolverConfig::new(dt, 0.7, 1).with_nonlinearity(nl);
    let mut state = AcousticState::new(grid);

    // Launch soliton towards the right wall at x = 49
    let mut packet = DirectionalWavePacket::new(30.0, 8.0, 2.5, 1.4, 0.0);
    packet.inject(&mut state, &material);

    let mut solver = AcousticSolver::new(state, material, boundary, config).unwrap();

    // Monitor probe at x = 35
    let probe = PressureProbe::new("rebound_sensor", 35, 8, 0);
    solver.add_probe(probe);

    // 1. Packet passes probe traveling rightwards (~10 steps)
    // 2. Hits right wall at x=49 and reflects leftwards (~30 steps)
    // 3. Passes probe traveling leftwards (~50 steps)
    for _ in 0..80 {
        solver.step();
    }

    let history = &solver.probes[0].history;
    let incident_peak = history.iter().take(25).map(|s| s.pressure).fold(0.0f32, f32::max);
    let reflected_peak = history.iter().skip(25).map(|s| s.pressure).fold(0.0f32, f32::max);

    assert!(incident_peak > 0.5, "Incident soliton should pass probe");
    assert!(reflected_peak > 0.5, "Reflected soliton should pass probe on return rebound");
}

#[test]
fn test_bohmian_quantum_potential_well_on_wavepacket() {
    let grid = GridConfig::new(40, 40, 1.0, 1.0);
    let mut state = AcousticState::new(grid);

    // Inject a smooth Gaussian wave packet centered at (20, 20)
    let impulse = GaussianImpulse::new(20.0, 20.0, 3.0, 2.0);
    impulse.inject(&mut state);

    // At the peak of a Gaussian envelope R(x,y), the curvature Laplacian(R) < 0,
    // so the Bohmian Quantum Potential Q = -Laplacian(R)/R > 0 at the peak,
    // while in the surrounding inflection ring Laplacian(R) > 0 => Q < 0.
    let q_center = state.quantum_potential_at(20, 20);
    let q_far = state.quantum_potential_at(5, 5);

    assert!(
        q_center > 0.1,
        "Bohmian quantum potential at Gaussian peak must reflect local envelope curvature (got {q_center})"
    );
    assert!(
        q_far.abs() < 1e-3,
        "Quantum potential far from excitation should vanish (got {q_far})"
    );

    // Verify quantum guidance force is non-zero on the flank of the wave packet
    let (fx, fy) = state.quantum_force_at(22, 20);
    assert!(
        fx.abs() > 0.01 && fy.abs() < 1e-5,
        "Quantum force on +X flank should be directed along X symmetry axis (fx={fx}, fy={fy})"
    );
}

#[test]
fn test_cohesion_erosion_under_acoustic_overpressure() {
    let grid = GridConfig::new(32, 16, 1.0, 1.0);
    let mut state = AcousticState::new(grid);
    let mut material = MaterialField::homogeneous(grid, 1.0, 1.0, 0.0);

    // Paint a cohesive solid region at [14..18, 6..10] with yield_pressure = 0.8
    material.set_cohesive_region(14, 18, 6, 10, 1.2, 1.2, 0.005, 1.0, 0.8);
    let idx_target = grid.index(15, 8);
    assert_eq!(material.cohesion[idx_target], 1.0);

    // Inject high-amplitude pressure wave (p = 3.5 > 0.8 yield) over the region
    let blast = GaussianImpulse::new(15.0, 8.0, 2.5, 3.5);
    blast.inject(&mut state);

    let boundary = BoundaryMask::new(grid, OuterBoundaryRule::RigidWall);
    let dt = SolverConfig::compute_cfl_dt(&grid, &material, 0.7);
    let config = SolverConfig::new(dt, 0.7, 1);
    let mut solver = AcousticSolver::new(state, material, boundary, config).unwrap();

    for _ in 0..8 {
        solver.step();
    }

    let eroded_cohesion = solver.material.cohesion[idx_target];
    assert!(
        eroded_cohesion < 1.0,
        "Overpressure exceeding yield threshold must erode material cohesion (got {eroded_cohesion})"
    );
}

#[test]
fn test_de_broglie_kerr_lensing_and_collimated_laser_fresnel_reflection() {
    let grid = GridConfig::new(64, 32, 1.0, 1.0);
    let mut state = AcousticState::new(grid);
    let mut material = MaterialField::homogeneous(grid, 1.0, 1.0, 0.01);
    let boundary = BoundaryMask::new(grid, OuterBoundaryRule::AbsorbingSponge);
    let dt = SolverConfig::compute_cfl_dt(&grid, &material, 0.7);

    // 1. Configure solver with de Broglie Kerr self-channeling (`kerr_lensing = 0.28`)
    //    and Bohm-Hiley internal velocity guidance (`bohmian_guidance = 0.02`)
    let nl = NonlinearLaw::new(0.45, 0.8, 0.03).with_de_broglie_guidance(0.28, 0.02);
    let config = SolverConfig::new(dt, 0.7, 1).with_nonlinearity(nl);
    let mut solver = AcousticSolver::new(state.clone(), material.clone(), boundary, config).unwrap();

    // Inject directional soliton and step 25 times; verify finite stability and stored bohmian_q
    let mut pkt = DirectionalWavePacket::new(14.0, 16.0, 2.0, 1.8, 0.0);
    pkt.inject(&mut solver.state, &solver.material);
    for _ in 0..25 {
        solver.step();
    }
    assert!(solver.state.validate().is_ok());
    let max_q = solver
        .state
        .bohmian_q
        .iter()
        .copied()
        .map(f32::abs)
        .fold(0.0f32, f32::max);
    assert!(
        max_q > 0.05,
        "Solver Pass 1 must continuously evaluate and store Bohmian Q in state.bohmian_q (got {max_q})"
    );

    // 2. Mini Sandbox: Collimated Laser Beam Fresnel Reflection & Crystal Fracture
    let mut bodies = vec![
        CohesiveBody::new_asteroid(1, 32.0, 16.0, 5.5, 6, 42, MineralClass::ObsidianAnvil),
        CohesiveBody::new_asteroid(2, 48.0, 16.0, 5.5, 6, 42, MineralClass::PiezoQuartz),
    ];
    let coh_before = bodies[0].mean_cohesion();

    // Fire laser along +X at the ObsidianAnvil asteroid at (32, 16)
    let laser_obs = CollimatedLaserBeam::new(10.0, 16.0, 0.0, 40.0, 1.2, 2.0);
    let strike_obs = laser_obs.fire_into_field(&mut state, &mut material, &mut bodies, false);
    assert_eq!(strike_obs.body_index, Some(0), "Laser must strike the ObsidianAnvil asteroid");
    assert!(
        strike_obs.reflection_coeff > 0.30,
        "High-impedance ObsidianAnvil (Z=2.02) must produce strong Fresnel reflection (got R={})",
        strike_obs.reflection_coeff
    );
    assert!(
        bodies[0].mean_cohesion() < coh_before,
        "Collimated laser strike must erode target crystal cohesion at the focal spot"
    );
    assert!(
        bodies[0].vx > 0.0,
        "Collimated laser radiation pressure must push struck asteroid along beam axis (+X)"
    );

    // Fire laser directly at the PiezoQuartz asteroid at (48, 16) and verify lower Fresnel reflection
    let laser_qtz = CollimatedLaserBeam::new(40.0, 16.0, 0.0, 20.0, 1.2, 2.0);
    let strike_qtz = laser_qtz.fire_into_field(&mut state, &mut material, &mut bodies, false);
    assert_eq!(strike_qtz.body_index, Some(1));
    assert!(
        strike_obs.reflection_coeff > strike_qtz.reflection_coeff,
        "ObsidianAnvil must reflect a larger fraction of laser energy than lower-impedance PiezoQuartz"
    );
}

