//! Corpus check: parse real-world URDF files and validate the results.
//!
//! Run with:
//! ```sh
//! URDF_CORPUS_DIR=~/workspace/urdf-corpus cargo test --test urdf_corpus -- --nocapture
//! ```
//!
//! Each file is parsed inside `catch_unwind`, so a parser panic is reported
//! per file instead of aborting the run. Files marked `expect_clean` must
//! parse, pass `RobotDescriptor::validate()` with zero problems, and survive
//! a forward-kinematics smoke test (`build` + `set_joint_position` +
//! `reset_joint_transforms`). Other files must at least not panic. The test
//! skips entirely when `URDF_CORPUS_DIR` is unset.

use std::str::FromStr;
use wgpu_robotic_simulator::urdf::RobotDescriptor;

struct Case {
    file: &'static str,
    /// Subdir of the corpus dir to use as CWD so relative mesh paths
    /// (e.g. kuka's `meshes/link_0.stl`) resolve.
    mesh_dir: Option<&'static str>,
    /// Parse + validate cleanly + FK smoke test must all pass.
    expect_clean: bool,
}

fn cases() -> Vec<Case> {
    vec![
        // ROS urdf_tutorial, in increasing complexity.
        Case { file: "01-myfirst.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "02-multipleshapes.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "03-origins.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "04-materials.urdf", mesh_dir: None, expect_clean: true },
        // .dae meshes: unsupported format -> warning + empty geometry.
        Case { file: "05-visual.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "06-flexible.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "07-physics.urdf", mesh_dir: None, expect_clean: true },
        // TurtleBot3: package:// + .dae meshes, xacro-namespaced elements,
        // top-level materials.
        Case { file: "turtlebot3_burger.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "turtlebot3_waffle.urdf", mesh_dir: None, expect_clean: true },
        Case { file: "turtlebot3_waffle_pi.urdf", mesh_dir: None, expect_clean: true },
        // Materials-only fragment: parses, but has no links.
        Case { file: "turtlebot3_common_properties.urdf", mesh_dir: None, expect_clean: false },
        // Kuka IIWA (7-DOF arm + gripper), meshes downloaded alongside.
        Case { file: "kuka_iiwa.urdf", mesh_dir: Some("kuka"), expect_clean: true },
        Case { file: "kuka_iiwa_free_base.urdf", mesh_dir: Some("kuka"), expect_clean: true },
        Case { file: "kuka_iiwa_vr_limits.urdf", mesh_dir: Some("kuka"), expect_clean: true },
        // The Rust urdf-rs crate's own sample file.
        Case { file: "urdf_rs_sample.urdf", mesh_dir: None, expect_clean: true },
    ]
}

fn fk_smoke(robot: &mut RobotDescriptor) -> Result<(), String> {
    let n = robot.joints.len();
    robot.build();
    // Mid-range-ish sweep: zeros, then ones (clamped to limits where set).
    for theta in [vec![0.0f32; n], vec![1.0f32; n]] {
        robot.set_joint_position(&theta, false);
        robot.build();
    }
    robot.reset_joint_transforms();
    robot.build();
    Ok(())
}

#[test]
fn corpus_check() {
    let corpus_dir = match std::env::var("URDF_CORPUS_DIR") {
        Ok(d) => std::path::PathBuf::from(d),
        Err(_) => {
            eprintln!("URDF_CORPUS_DIR unset; skipping corpus check");
            return;
        }
    };
    if !corpus_dir.is_dir() {
        eprintln!("URDF_CORPUS_DIR={} is not a dir; skipping", corpus_dir.display());
        return;
    }

    let mut failures = Vec::new();
    for case in cases() {
        let path = corpus_dir.join(case.file);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                failures.push(format!("{}: cannot read file: {e}", case.file));
                continue;
            }
        };
        // Resolve relative mesh paths against the case's mesh dir.
        let saved_cwd = std::env::current_dir().ok();
        if let Some(md) = case.mesh_dir {
            if let Err(e) = std::env::set_current_dir(corpus_dir.join(md)) {
                failures.push(format!("{}: cannot chdir to {md}: {e}", case.file));
                continue;
            }
        }

        let parsed = std::panic::catch_unwind(|| RobotDescriptor::from_str(&text));

        if let Some(cwd) = saved_cwd {
            let _ = std::env::set_current_dir(cwd);
        }

        let mut robot = match parsed {
            Err(_) => {
                let msg = format!("{}: PARSER PANIC", case.file);
                println!("{msg}");
                failures.push(msg);
                continue;
            }
            Ok(Err(e)) => {
                let msg = format!("{}: parse error: {e}", case.file);
                println!("{msg}");
                if case.expect_clean {
                    failures.push(msg);
                }
                continue;
            }
            Ok(Ok(r)) => r,
        };

        let problems = robot.validate();
        let n_visuals: usize = robot.links.iter().map(|l| l.visuals.len()).sum();
        let n_collisions: usize = robot.links.iter().map(|l| l.collisions.len()).sum();
        println!(
            "{}: {} links, {} joints, {} visuals, {} collisions{}",
            case.file,
            robot.links.len(),
            robot.joints.len(),
            n_visuals,
            n_collisions,
            if problems.is_empty() { "" } else { " <-- PROBLEMS" },
        );
        for p in &problems {
            println!("    ! {p}");
        }

        let smoke = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fk_smoke(&mut robot)));
        match smoke {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let msg = format!("{}: FK smoke failed: {e}", case.file);
                println!("    ! {msg}");
                failures.push(msg);
            }
            Err(_) => {
                let msg = format!("{}: FK smoke PANIC", case.file);
                println!("    ! {msg}");
                failures.push(msg);
            }
        }

        if case.expect_clean && !problems.is_empty() {
            failures.push(format!(
                "{}: expected clean, got {} problem(s)",
                case.file,
                problems.len()
            ));
        }
    }

    println!("\n{} file(s), {} failure(s)", cases().len(), failures.len());
    for f in &failures {
        println!("  FAIL: {f}");
    }
    assert!(failures.is_empty(), "corpus check failed");
}
