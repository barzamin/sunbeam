use anyhow::Result;
use clap::Parser;
use image::ColorType;
use std::path::PathBuf;
use ultraviolet::Vec3;

mod random;
mod trace;
use random::UniformInSphere;
use trace::{Ray, Probe, Sphere, Scene};

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
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        let viewport_height = 2.;
        let viewport_width = aspect * viewport_height;
        let focal_length = 1.0;

        let origin: Vec3 = (0., 0., 0.).into();
        let horiz: Vec3 = (viewport_width, 0., 0.).into();
        let vert: Vec3 = (0., viewport_height, 0.).into();
        let lower_left = origin - horiz / 2. - vert / 2. - (0., 0., focal_length).into();

        Self {
            origin, horiz, vert, lower_left
        }
    }

    pub fn ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(self.origin, self.lower_left + u * self.horiz + v * self.vert - self.origin)
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

    let camera = Camera::new(aspect);

    // -- camera
    let mut scene = Scene::new();
    scene.add(Box::new(Sphere {
        center: (0., 0., -1.).into(),
        radius: 0.5,
    }));
    scene.add(Box::new(Sphere {
        center: (0., -100.5, -1.).into(),
        radius: 100.,
    }));

    // -- render
    for i in 0..fb.height {
        // render scanline
        for j in 0..fb.width {
            let u = j as f32 / (fb_width - 1) as f32;
            let v = 1. - i as f32 / (fb_height - 1) as f32;

            let ray = camera.ray(u, v);
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
