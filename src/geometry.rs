use crate::material::*;
use nalgebra::Vector3;

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
    pub material: &'a Box<dyn Material>,
}

pub trait Hittable: Send + Sync {
    fn hits(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Vector3<f32>,
    pub radius: f32,
    pub material: Box<dyn Material>,
}

impl Hittable for Sphere {
    fn hits(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // t^2 + 2 * t(axis * direction) * t + axis * axis = radius^2; solve for t.
        let axis = ray.origin() - self.center;
        let b = 2.0 * (axis.dot(ray.direction()));
        let c = axis.dot(&axis) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * c;
        if discriminant >= 0.0 {
            // Return the first intersection in the relevant range.
            for sign in [-1.0, 1.0].into_iter() {
                let t = (-b + discriminant.sqrt() * sign) / 2.0;
                if t_min <= t && t < t_max {
                    let pos = ray.at(t);
                    let normal = (pos - self.center).normalize();
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
pub struct HittableList {
    pub hittables: Vec<Box<dyn Hittable>>,
}

impl Hittable for HittableList {
    fn hits(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // There's probably a nicer way to do this with iterators, but I can't
        // quite find it.
        let mut best_rec: Option<HitRecord> = None;
        for rec in self.hittables.iter().map(|h| h.hits(ray, t_min, t_max)) {
            if let Some(rec) = rec {
                if rec.t < best_rec.as_ref().map_or(std::f32::INFINITY, |r| r.t) {
                    best_rec = Some(rec);
                }
            }
        }
        best_rec
    }
}

/// The camera's render plane is located at `lower_left`, with `horizontal` and
/// `vertical` representing the axes. `origin` is where the camera is looking
/// from.
pub struct Camera {
    pub lower_left: Vector3<f32>,
    pub horizontal: Vector3<f32>,
    pub vertical: Vector3<f32>,
    pub origin: Vector3<f32>,
}

impl Camera {
    /// The relevant ray to render. u and v are coordinates in screenspace, where
    /// 1.0 is the farthest along that axis (i.e., they will *not* have the same
    /// scale).
    pub fn ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}

/// Samples a random point on the unit sphere. Note that this is *not* the same
/// as `rand::distributions::UnitSphereSurface`, which samples from the
/// *surface*.
#[derive(Clone, Copy, Debug)]
pub struct UnitSphere;

impl rand::distributions::Distribution<[f32; 3]> for UnitSphere {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> [f32; 3] {
        loop {
            let x = rng.gen();
            let y = rng.gen();
            let z = rng.gen();
            if x * x + y * y + z * z <= 1.0 {
                return [x, y, z];
            }
        }
    }
}
