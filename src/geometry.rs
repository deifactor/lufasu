use crate::material::*;
use enum_dispatch::enum_dispatch;
use nalgebra::Vector3;

#[derive(Debug)]
pub struct Ray {
    origin: Vector3<f32>,
    // Must be normalized.
    direction: Vector3<f32>,
}

// Coordinate system is: x is right, y is up, z is *towards* the viewer.

impl Ray {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
        debug_assert!(direction.magnitude() != 0.0);
        Ray {
            origin,
            direction: direction.normalize(),
        }
    }
    pub fn origin(&self) -> &Vector3<f32> {
        &self.origin
    }
    pub fn direction(&self) -> &Vector3<f32> {
        &self.direction
    }
    pub fn at(&self, t: f32) -> Vector3<f32> {
        self.origin + t * self.direction
    }
}

pub struct HitRecord<'a> {
    // The `t` for the ray. Guaranteed to satisfy `t_min <= t < t_max`.
    pub t: f32,
    // The position in worldspace where the ray hit the object.
    pub pos: Vector3<f32>,
    // Surface normal at the hitpoint. This is optional because some objects,
    // like fog, can be hit but don't have normals.
    pub normal: Option<Vector3<f32>>,
    pub material: &'a MaterialEnum,
}

#[enum_dispatch]
pub trait Hittable: std::fmt::Debug + Send + Sync {
    fn hits(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[enum_dispatch(Hittable)]
#[derive(Debug)]
pub enum HittableEnum {
    Sphere,
    HittableList,
}

#[derive(Debug)]
pub struct Sphere {
    pub center: Vector3<f32>,
    pub radius: f32,
    pub material: MaterialEnum,
}

impl Hittable for Sphere {
    fn hits(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // t^2 + 2 * t(axis * direction) * t + axis * axis = radius^2; solve for t.
        let axis = ray.origin() - self.center;
        let b = 2.0 * axis.dot(ray.direction());
        let c = axis.dot(&axis) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * c;
        if discriminant >= 0.0 {
            // Return the first intersection in the relevant range.
            for sign in [-1.0, 1.0].into_iter() {
                let t = (-b + discriminant.sqrt() * sign) / 2.0;
                if t_min <= t && t < t_max {
                    let pos = ray.at(t);
                    let normal = (pos - self.center) / self.radius;
                    return Some(HitRecord {
                        t,
                        pos,
                        normal: Some(normal),
                        material: &self.material,
                    });
                }
            }
        }
        None
    }
}

// A convenient way to hit-test a bunch of objects. The returned hit is the
// first hitpoint among any elements.
#[derive(Debug)]
pub struct HittableList {
    pub hittables: Vec<HittableEnum>,
}

impl Hittable for HittableList {
    fn hits(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // There's probably a nicer way to do this with iterators, but I can't
        // quite find it.
        let mut best_rec: Option<HitRecord> = None;
        let mut best_t = t_max;
        for hittable in &self.hittables {
            if let Some(rec) = hittable.hits(ray, t_min, best_t) {
                if rec.t < best_t {
                    best_t = rec.t;
                    best_rec = Some(rec);
                }

            }
        }
        best_rec
    }
}

// The camera. Used to compute direction of rays and so on.
pub struct Camera {
    lower_left: Vector3<f32>,
    horizontal: Vector3<f32>,
    vertical: Vector3<f32>,
    origin: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    lens_radius: f32,
}

impl Camera {
    pub fn new(
        origin: Vector3<f32>,
        look_at: Vector3<f32>,
        up: Vector3<f32>,
        vertical_fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
    ) -> Self {
        let half_height = (vertical_fov / 2.0).tan();
        let half_width = aspect_ratio * half_height;
        let w = (origin - look_at).normalize();
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);
        let lower_left = origin - focus_distance * (half_width * u + half_height * v + w);
        Self {
            lower_left,
            horizontal: 2.0 * half_width * focus_distance * u,
            vertical: 2.0 * half_height * focus_distance * v,
            origin,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    /// The relevant ray to render. u and v are coordinates in screenspace, where
    /// 1.0 is the farthest along that axis (i.e., they will *not* have the same
    /// scale).
    pub fn ray<R: rand::Rng + ?Sized>(&self, s: f32, t: f32, rng: &mut R) -> Ray {
        let lens_position: [f32; 2] = rng.sample(rand_distr::UnitDisc);
        let lens_offset =
            self.lens_radius * (lens_position[0] * self.u + lens_position[1] * self.v);
        Ray::new(
            self.origin + lens_offset,
            self.lower_left + s * self.horizontal + t * self.vertical - self.origin - lens_offset,
        )
    }
}
