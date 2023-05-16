use anyhow::Result;
use clap::Parser;
use image::ColorType;
use std::path::PathBuf;
use ultraviolet::Vec3;

mod random;
use random::UniformInSphere;

type Color = Vec3;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, value_name = "IMAGE", default_value = "render.png")]
    out: PathBuf,
}

struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buf: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buf: vec![0; width * height * 3],
        }
    }

    pub fn write(&mut self, i: usize, j: usize, color: Color) {
        let p = (i * self.width + j) * 3;
        self.buf[p + 0] = (255. * color.x) as u8;
        self.buf[p + 1] = (255. * color.y) as u8;
        self.buf[p + 2] = (255. * color.z) as u8;
    }
}

#[derive(Debug)]
struct Ray {
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
struct Hit {
    pub p: Vec3,
    pub t: f32,
    pub normal: Vec3,
}

impl Hit {
    pub fn front_facing(&self, ray: &Ray) -> bool {
        self.normal.dot(ray.dir) > 0.
    }
}

trait Probe {
    fn probe(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit>;
}

struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Probe for Sphere {
    fn probe(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let sep = ray.origin - self.center;
        let a = ray.dir.mag_sq();
        let hb = sep.dot(ray.dir);
        let c = sep.dot(sep) - self.radius * self.radius;
        let discrim = hb*hb - a*c;

        if discrim < 0. {
            return None;
        }
        let sqd = discrim.sqrt();

        let mut root = (-hb-sqd)/a;
        if root < t_min || t_max < root {
            root = (-hb+sqd)/a;
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

struct Scene {
    objects: Vec<Box<dyn Probe>>,
}
impl Scene {
    pub fn new() -> Self {
        Self {
            objects: vec![]
        }
    }

    pub fn add(&mut self, object: Box<dyn Probe>) {
        self.objects.push(object);
    }

    pub fn probe(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let mut closest = t_max;
        let mut current_hit = None;

        for object in &self.objects {
            if let Some(hit) = object.probe(ray, t_min, closest) {
                current_hit = Some(hit);
                closest = hit.t;
            }
        }

        current_hit
    }
}

fn color_ray(ray: &Ray, scene: &Scene) -> Color {
    if let Some(hit) = scene.probe(ray, 0., f32::INFINITY) {
        return 0.5 * (hit.normal + Vec3::one());
    }

    let t = 0.5 * (ray.dir.normalized().y + 1.);
    (1. - t) * Color::one() + t * Color::new(0.5, 0.7, 1.0)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // -- framebuffer
    let aspect = 16. / 9.;
    let fb_width = 400;
    let fb_height = (fb_width as f32 / aspect) as usize;
    let mut fb = Framebuffer::new(fb_width, fb_height);

    // -- camera
    let viewport_height = 2.;
    let viewport_width = aspect * viewport_height;
    let focal_length = 1.0;

    let origin: Vec3 = (0., 0., 0.).into();
    let horiz: Vec3 = (viewport_width, 0., 0.).into();
    let vert: Vec3 = (0., viewport_height, 0.).into();
    let lower_left = origin - horiz / 2. - vert / 2. - (0., 0., focal_length).into();

    let mut scene = Scene::new();
    scene.add(Box::new(Sphere { center: (0., 0., -1.).into(), radius: 0.5}));

    // -- render
    for i in 0..fb.height {
        // render scanline
        for j in 0..fb.width {
            let u = j as f32 / (fb_width - 1) as f32;
            let v = 1. - i as f32 / (fb_height - 1) as f32;

            let ray = Ray::new(origin, lower_left + u * horiz + v * vert - origin);
            fb.write(i, j, color_ray(&ray, &scene));
        }
    }

    image::save_buffer(
        args.out,
        &fb.buf,
        fb.width as u32,
        fb.height as u32,
        ColorType::Rgb8,
    )?;

    Ok(())
}
