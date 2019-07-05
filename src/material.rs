use crate::geometry::*;
use nalgebra::Vector3;
use rand::prelude::*;

pub struct Scattering {
    /// The outgoing ray should have its color multiplied by this factor.
    pub attenuation: palette::LinSrgb,
    /// The new traced ray.
    pub scattered: Ray,
}

pub trait Material: Send + Sync {
    /// If this returns None, the ray did not scatter at all. Useful for
    /// transparent materials, fogs, etc.
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut dyn rand::RngCore,
    ) -> Option<Scattering>;
}

pub struct Lambertian {
    pub albedo: palette::LinSrgb,
}

impl Material for Lambertian {
    fn scatter(
        &self,
        _ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut dyn rand::RngCore,
    ) -> Option<Scattering> {
        let direction = hit_record.normal.unwrap() + Vector3::<f32>::from(rng.sample(UnitSphere));
        Some(Scattering {
            attenuation: self.albedo,
            scattered: Ray::new(hit_record.pos, direction),
        })
    }
}

pub struct Metal {
    pub albedo: palette::LinSrgb,
}

impl Material for Metal {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        _rng: &mut dyn rand::RngCore,
    ) -> Option<Scattering> {
        let normal = hit_record.normal.unwrap();
        let reflected = ray.direction() - (2.0 * ray.direction().dot(&normal)) * normal;
        let scattered = Ray::new(hit_record.pos, reflected);
        if scattered.direction().dot(&normal) > 0.0 {
            Some(Scattering {
                attenuation: self.albedo,
                scattered,
            })
        } else {
            None
        }
    }
}
