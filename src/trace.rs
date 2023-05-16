use std::sync::Arc;

use crate::{material::Material, Color};
use ultraviolet::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.dir * t
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub p: Vec3,
    pub t: f32,
    pub normal: Vec3,
}

impl Hit {
    pub fn front_facing(&self, ray: &Ray) -> bool {
        self.normal.dot(ray.dir) > 0.
    }
}

pub trait Probe {
    fn probe(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit>;
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Probe for Sphere {
    fn probe(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let sep = ray.origin - self.center;
        let a = ray.dir.mag_sq();
        let hb = sep.dot(ray.dir);
        let c = sep.dot(sep) - self.radius * self.radius;
        let discrim = hb * hb - a * c;

        if discrim < 0. {
            return None;
        }
        let sqd = discrim.sqrt();

        let mut root = (-hb - sqd) / a;
        if root < t_min || t_max < root {
            root = (-hb + sqd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let p = ray.at(root);
        Some(Hit {
            t: root,
            p,
            normal: (p - self.center) / self.radius,
        })
    }
}

pub struct Scene {
    objects: Vec<Box<dyn Probe>>,
    materials: Vec<Arc<dyn Material>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            materials: vec![],
        }
    }

    pub fn add(&mut self, object: Box<dyn Probe>, material: Arc<dyn Material>) {
        self.objects.push(object);
        self.materials.push(material);
    }

    pub fn probe(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(Hit, &dyn Material)> {
        let mut closest = t_max;
        let mut current_hit = None;

        for (object, material) in self.objects.iter().zip(&self.materials) {
            if let Some(hit) = object.probe(ray, t_min, closest) {
                current_hit = Some((hit, material.as_ref()));
                closest = hit.t;
            }
        }

        current_hit
    }
}
