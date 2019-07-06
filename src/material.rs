use crate::geometry::*;
use enum_dispatch::enum_dispatch;
use nalgebra::Vector3;
use palette::LinSrgb;
use rand::prelude::*;
use rand_distr;

pub struct Scattering {
    /// The outgoing ray should have its color multiplied by this factor.
    pub attenuation: palette::LinSrgb,
    /// The new traced ray.
    pub scattered: Ray,
}

#[enum_dispatch]
pub trait Material: std::fmt::Debug + Send + Sync {
    /// If this returns None, the ray did not scatter at all. Useful for
    /// transparent materials, fogs, etc.
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut dyn rand::RngCore,
    ) -> Option<Scattering>;
}

#[enum_dispatch(Material)]
#[derive(Debug)]
pub enum MaterialEnum {
    Lambertian,
    Dielectric,
    Metal,
}

#[derive(Debug)]
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
        let direction =
            hit_record.normal.unwrap() + Vector3::<f32>::from(rng.sample(rand_distr::UnitSphere));
        Some(Scattering {
            attenuation: self.albedo,
            scattered: Ray::new(hit_record.pos, direction),
        })
    }
}

#[derive(Debug)]
pub struct Metal {
    pub albedo: palette::LinSrgb,
    pub fuzz: f32,
}

impl Material for Metal {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut dyn rand::RngCore,
    ) -> Option<Scattering> {
        let normal = hit_record.normal.unwrap();
        let reflected = reflect(ray.direction(), &normal)
            + Vector3::<f32>::from(rng.sample(rand_distr::UnitSphere)) * self.fuzz;
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

#[derive(Debug)]
pub struct Dielectric {
    pub index: f32,
}

impl Dielectric {
    fn schlick(&self, cosine: f32) -> f32 {
        let r0 = ((1.0 - self.index) / (1.0 + self.index)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray: &Ray,
        hit_record: &HitRecord,
        rng: &mut dyn rand::RngCore,
    ) -> Option<Scattering> {
        let attenuation = LinSrgb::new(1.0, 1.0, 1.0);
        let normal = hit_record.normal.unwrap();
        let (outward_normal, index_ratio, cosine) = if ray.direction().dot(&normal) > 0.0 {
            (
                -normal,
                self.index,
                self.index * ray.direction().dot(&normal),
            )
        } else {
            (normal, 1.0 / self.index, -ray.direction().dot(&normal))
        };
        let direction =
            if let Some(refracted) = refract(ray.direction(), &outward_normal, index_ratio) {
                if rng.gen_bool(f64::from(self.schlick(cosine))) {
                    reflect(ray.direction(), &normal)
                } else {
                    refracted
                }
            } else {
                reflect(ray.direction(), &normal)
            };
        Some(Scattering {
            attenuation,
            scattered: Ray::new(hit_record.pos, direction),
        })
    }
}

fn reflect(v: &Vector3<f32>, normal: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(normal) * normal
}

fn refract(v: &Vector3<f32>, normal: &Vector3<f32>, index_ratio: f32) -> Option<Vector3<f32>> {
    let v_norm = v.normalize();
    let cos_theta = v_norm.dot(normal);
    let discriminant = 1.0 - index_ratio * index_ratio * (1.0 - cos_theta * cos_theta);
    if discriminant > 0.0 {
        Some(index_ratio * (v_norm - cos_theta * normal) - normal * discriminant.sqrt())
    } else {
        None
    }
}
