use anyhow::Result;
use clap::Parser;
use image::ColorType;
use indicatif::ProgressBar;
use material::{Dielectric, Lambertian, Metallic, ScatteringResult};
use rand::Rng;
use std::{path::PathBuf, sync::Arc};
use ultraviolet::Vec3;

mod material;
mod random;
mod trace;
use random::{UniformInDisc, UniformInSphere, UniformOnSphere};
use trace::{Probe, Ray, Scene, Sphere};

pub(crate) type Color = Vec3;

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

struct Camera {
    origin: Vec3,
    horiz: Vec3,
    vert: Vec3,
    lower_left: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    lens_radius: f32,
}

impl Camera {
    pub fn new(
        aspect: f32,
        fovy: f32,
        aperture: f32,
        focus_dist: f32,
        from: Vec3,
        to: Vec3,
        up: Vec3,
    ) -> Self {
        let fovy = fovy * core::f32::consts::PI / 180.;
        let viewport_height = 2. * (fovy / 2.).tan();
        let viewport_width = aspect * viewport_height;

        let w = (from - to).normalized();
        let u = up.cross(w).normalized();
        let v = w.cross(u);

        let origin: Vec3 = from;
        let horiz: Vec3 = u * viewport_width * focus_dist;
        let vert: Vec3 = v * viewport_height * focus_dist;
        let lower_left = origin - horiz / 2. - vert / 2. - focus_dist*w;

        Self {
            origin,
            horiz,
            vert,
            lower_left,
            u,
            v,
            w,
            lens_radius: aperture / 2.,
        }
    }

    pub fn ray(&self, u: f32, v: f32) -> Ray {
        let lenspos = self.lens_radius * rand::thread_rng().sample(UniformInDisc);
        let offset = self.u * lenspos.x + self.v * lenspos.y;

        Ray::new(
            self.origin + offset,
            self.lower_left + u * self.horiz + v * self.vert - self.origin - offset,
        )
    }
}

fn color_ray(incoming_ray: &Ray, scene: &Scene, depth: usize, log: bool) -> Color {
    if depth <= 0 {
        return Color::zero();
    }

    if let Some((hit, material)) = scene.probe(incoming_ray, 0.001, f32::INFINITY) {
        if log {
            println!(
                "hit {:?} on mat {:?} ({}) from incoming {:?}",
                hit,
                material,
                if hit.front(incoming_ray) {
                    "outside"
                } else {
                    "inside"
                },
                incoming_ray
            );
        }
        if let ScatteringResult::Scattered { ray, attenuation } =
            material.scatter(incoming_ray, &hit)
        {
            if log {
                println!("  -> scattered to {:?} with atten {:?}", ray, attenuation);
            }
            return attenuation * color_ray(&ray, scene, depth - 1, log);
        }

        return Color::zero();
    }

    let t = 0.5 * (incoming_ray.dir.normalized().y + 1.);
    (1. - t) * Color::one() + t * Color::new(0.5, 0.7, 1.0)
}

fn construct_test_scene() -> Scene {
    let mut scene = Scene::new();

    let material1 = Arc::new(Lambertian::new((0.3, 0.2, 0.8).into()));
    let material2 = Arc::new(Lambertian::new((0.8, 0.8, 0.0).into()));
    // let material3 = Arc::new(Metallic::new((0.8, 0.8, 0.8).into(), 0.3));
    let material3 = Arc::new(Dielectric::new(1.5));
    let material4 = Arc::new(Metallic::new((0.8, 0.6, 0.2).into(), 0.));
    scene.add(
        Box::new(Sphere {
            center: (0., 0., -1.).into(),
            radius: 0.5,
        }),
        material1.clone(),
    );
    scene.add(
        Box::new(Sphere {
            center: (0., -100.5, -1.).into(),
            radius: 100.,
        }),
        material2.clone(),
    );
    scene.add(
        Box::new(Sphere {
            center: (-1., 0., -1.).into(),
            radius: 0.5,
        }),
        material3.clone(),
    );
    scene.add(
        Box::new(Sphere {
            center: (1., 0., -1.).into(),
            radius: 0.5,
        }),
        material4.clone(),
    );

    scene
}

fn main() -> Result<()> {
    let args = Args::parse();

    // -- framebuffer
    let aspect = 16. / 9.;
    let fb_width = 400;
    let fb_height = (fb_width as f32 / aspect) as usize;
    let mut fb = Framebuffer::new(fb_width, fb_height);

    let from = Vec3::new(3., 3., 2.);
    let to = Vec3::new(0., 0., -1.);
    let focal_plane = (to - from).mag();
    let aperture = 2.;
    let camera = Camera::new(
        aspect,
        20.,
        aperture,
        focal_plane,
        from,
        to,
        (0., 1., 0.).into(),
    );

    // -- camera
    let scene = construct_test_scene();

    let mut rng = rand::thread_rng();
    let supersamples = 32;

    // -- render
    let pb = ProgressBar::new(fb.height as u64);
    for i in 0..fb.height {
        // render scanline
        for j in 0..fb.width {
            let mut color = Color::zero();
            for _ in 0..supersamples {
                let u = (j as f32 + rng.gen::<f32>()) / (fb_width - 1) as f32;
                let v = 1. - (i as f32 + rng.gen::<f32>()) / (fb_height - 1) as f32;

                let ray = camera.ray(u, v);
                color += color_ray(&ray, &scene, 40, false);
            }
            color /= supersamples as f32;
            color.apply(|x| x.powf(1. / 2.2));
            fb.write(i, j, color);
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
