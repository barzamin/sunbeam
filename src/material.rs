use rand::Rng;
use std::fmt::Debug;
use ultraviolet::Vec3;

use crate::random::{UniformInSphere, UniformOnSphere};
use crate::trace::{Hit, Ray};
use crate::Color;

pub enum ScatteringResult {
    Scattered { ray: Ray, attenuation: Color },
    Absorbed,
}
pub trait Material: Debug {
    fn scatter(&self, incoming_ray: &Ray, hit: &Hit) -> ScatteringResult;
}

#[derive(Debug)]
pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, incoming_ray: &Ray, hit: &Hit) -> ScatteringResult {
        let S = rand::thread_rng().sample(UniformOnSphere);
        let mut scatter_dir = hit.normal + S;
        if scatter_dir.mag() < 0.00000001 {
            scatter_dir = hit.normal;
        }

        ScatteringResult::Scattered {
            ray: Ray::new(hit.p, scatter_dir),
            attenuation: self.albedo,
        }
    }
}

#[derive(Debug)]
pub struct Metallic {
    albedo: Color,
    roughness: f32,
}

impl Metallic {
    pub fn new(albedo: Color, roughness: f32) -> Self {
        Self { albedo, roughness }
    }
}

impl Material for Metallic {
    fn scatter(&self, incoming_ray: &Ray, hit: &Hit) -> ScatteringResult {
        let mut refl = incoming_ray.dir.normalized().reflected(hit.normal);
        refl += self.roughness * rand::thread_rng().sample(UniformInSphere);

        if refl.dot(hit.normal) < 0. {
            ScatteringResult::Absorbed
        } else {
            ScatteringResult::Scattered {
                ray: Ray::new(hit.p, refl),
                attenuation: self.albedo,
            }
        }
    }
}

#[derive(Debug)]
pub struct Dielectric {
    ior: f32,
}

impl Dielectric {
    pub fn new(ior: f32) -> Self {
        Self { ior }
    }
}

fn refract(v: Vec3, normal: Vec3, eta: f32) -> Vec3 {
    let cos_theta = (-v).dot(normal).min(1.);
    let r_out_prp = eta * (v + cos_theta * normal);
    let r_out_par = -(1. - r_out_prp.mag_sq()).abs().sqrt() * normal;

    r_out_prp + r_out_par
}

fn reflectance(v: Vec3, normal: Vec3, eta: f32) -> f32 {
    let cosine = (-normal).dot(v).min(1.);
    let mut r0 = (1.-eta) / (1.+eta);
    r0 = r0*r0;
    r0 + (1.-r0)*(1. - cosine).powf(5.)
}

impl Material for Dielectric {
    fn scatter(&self, incoming_ray: &Ray, hit: &Hit) -> ScatteringResult {
        let attenuation = Color::one();
        let ratio = if hit.front(incoming_ray) {
            1. / self.ior
        } else {
            self.ior
        };
        // make the normal always face outward
        let n = if hit.front(incoming_ray) {
            hit.normal
        } else {
            -hit.normal
        };

        // let refracted = refract(incoming_ray.dir.normalized(), n, ratio);
        let mut scatter = incoming_ray.dir.normalized().refracted(n, ratio);
        if scatter.abs().component_max() <= f32::EPSILON
            || reflectance(incoming_ray.dir.normalized(), n, ratio)
                >= rand::thread_rng().gen::<f32>()
        {
            scatter = incoming_ray.dir.normalized().reflected(hit.normal);
        }

        ScatteringResult::Scattered {
            ray: Ray::new(hit.p, scatter),
            attenuation,
        }
    }
}
