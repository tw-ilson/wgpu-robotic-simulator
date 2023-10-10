use glm::{Vec2, Vec3};
use nalgebra_glm as glm;
use rand::distributions::{Distribution, Standard};
use rand::prelude::*;
use rand::Rng;
use std::{thread, time};

pub trait PhysicsProgram {
    fn new() -> Self;
    fn setup(&mut self, scene: &str);
    fn step(&mut self);
    fn apply_forces(&mut self);
    fn update_kinematics(&mut self);
    fn detect_collisions(&mut self);
    fn solve_constraints(&mut self);
}

pub struct FreeBody {
    // linear kinematic info
    posn: Vec2,
    vel: Vec2,
    acc: Vec2,

    //angular kinematic info
    theta: f32,
    vel_theta: f32,
    acc_theta: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Particle {
    posn: Vec2,
    // accel: Vec2,
    vel: Vec2,
    mass: f32,
}
impl Distribution<Particle> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle {
        Particle {
            posn: glm::vec2(rng.gen(), rng.gen()),
            vel: glm::vec2(0.0, 0.0),
            // accel: glm::vec2(0, 0),
            mass: rng.gen(),
        }
    }
}

impl Particle {
    fn compute_gravity(&self) -> Vec2 {
        glm::vec2(0.0, -9.8 * self.mass)
    }
}

#[derive(Clone, Debug)]
pub struct ParticleSim(Vec<Particle>);

impl PhysicsProgram for ParticleSim {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn setup(&mut self, scene: &str) {
        let mut rng = rand::thread_rng();
        let ParticleSim(particles) = self;
        let n = 100;
        for _ in 0..n {
            particles.push(rng.gen());
        }
    }
    fn step(&mut self) {
        let ParticleSim(particles) = self;
        let t_final = 10.0;
        let dt = 1;
        let mut t = 0.0;

        while t < t_final {
            std::thread::sleep(time::Duration::from_secs(dt));
            for p in particles.iter_mut() {
                let fg: Vec2 = p.compute_gravity();
                let a: Vec2 = fg / p.mass;
                p.vel += a * (dt as f32);
                p.posn += p.vel * (dt as f32);
            }
            t += 1.;
        }
        println!("particles: {:?}", self);
    }
    fn apply_forces(&mut self) {}
    fn update_kinematics(&mut self) {}
    fn detect_collisions(&mut self) {}
    fn solve_constraints(&mut self) {}
}
