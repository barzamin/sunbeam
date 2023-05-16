use rand::Rng;

use crate::random::UniformOnSphere;
use crate::trace::{Hit, Ray};
use crate::Color;

pub enum ScatteringResult {
    Scattered { ray: Ray, attenuation: Color },
    Absorbed,
}
pub trait Material {
    fn scatter(&self, incoming_ray: &Ray, hit: &Hit) -> ScatteringResult;
}

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

pub struct Metallic {
    albedo: Color,
}

impl Metallic {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metallic {
    fn scatter(&self, incoming_ray: &Ray, hit: &Hit) -> ScatteringResult {
        let refl = incoming_ray.dir.normalized().reflected(hit.normal);

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
