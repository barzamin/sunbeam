use rand::distributions::Distribution;
use rand_distr::{Normal, Uniform};
use ultraviolet::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct UniformInSphere;

impl Distribution<Vec3> for UniformInSphere {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let normal = Normal::<f32>::new(0., 1.).unwrap();
        let uniform = Uniform::<f32>::new(0., 1.);

        let u = uniform.sample(rng);
        let mut p = Vec3::new(normal.sample(rng), normal.sample(rng), normal.sample(rng));
        p.normalize();
        p /= u.cbrt();

        p
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UniformOnSphere;

impl Distribution<Vec3> for UniformOnSphere {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let normal = Normal::<f32>::new(0., 1.).unwrap();

        let mut p = Vec3::new(normal.sample(rng), normal.sample(rng), normal.sample(rng));
        p.normalize();

        p
    }
}
