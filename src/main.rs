mod geometry;
mod material;

use itertools::iproduct;
use minifb::{Window, WindowOptions};
use nalgebra::Vector3;
use palette::{LinSrgb, Mix, Srgb};
use rand::distributions::Uniform;
use rand::prelude::*;
use rayon::prelude::*;
use std::error::Error;

use geometry::*;
use material::*;

const WIDTH: usize = 200;
const HEIGHT: usize = 100;
// Number of per-pixel samples.
const SAMPLE_COUNT: usize = 100;
// Maximum number of bounces to use. After this, we assume the ray will be
// black.
const BOUNCES: usize = 50;

pub fn color<T: Hittable, R: rand::Rng>(
    ray: &Ray,
    world: &T,
    bounce: usize,
    rng: &mut R,
) -> LinSrgb {
    if let Some(hit_record) = world.hits(ray, 0.001, std::f32::INFINITY) {
        if bounce < BOUNCES {
            if let Some(scattering) = hit_record.material.scatter(ray, &hit_record, rng) {
                return scattering.attenuation
                    * color(&scattering.scattered, world, bounce + 1, rng);
            }
        }
        return LinSrgb::new(0.0, 0.0, 0.0);
    } else {
        let t = (ray.direction().y + 1.0) / 2.0;
        let white = LinSrgb::new(1.0, 1.0, 1.0);
        let blue = LinSrgb::new(0.5, 0.7, 1.0);
        white.mix(&blue, t as f32)
    }
}

fn construct_scene<R: rand::Rng>(rng: &mut R) -> HittableList {
    let spheres = iproduct!(-11..11, -11..11).filter_map(|(x, z)| -> Option<HittableEnum> {
        let center = Vector3::<f32>::new(
            (x as f32) + rng.sample(Uniform::new(0.0, 0.9)),
            0.2,
            (z as f32) + rng.sample(Uniform::new(0.0, 0.9)),
        );
        if (center - Vector3::new(4.0, 0.2, 0.0)).norm() > 0.9 {
            let material_choice: f32 = rng.gen();
            let material = if material_choice < 0.8 {
                // Diffuse.
                MaterialEnum::from(Lambertian {
                    albedo: LinSrgb::new(
                        rng.gen::<f32>().powi(2),
                        rng.gen::<f32>().powi(2),
                        rng.gen::<f32>().powi(2),
                    ),
                })
            } else if material_choice < 0.95 {
                // Metal.
                MaterialEnum::from(Metal {
                    albedo: LinSrgb::new(
                        rng.sample(Uniform::new(0.5, 1.0)),
                        rng.sample(Uniform::new(0.5, 1.0)),
                        rng.sample(Uniform::new(0.5, 1.0)),
                    ),
                    fuzz: rng.sample(Uniform::new(0.0, 0.5)),
                })
            } else {
                MaterialEnum::from(Dielectric { index: 1.5 })
            };
            Some(
                Sphere {
                    center,
                    radius: 0.2,
                    material,
                }
                .into(),
            )
        } else {
            None
        }
    });
    let others: Vec<Sphere> = vec![
        Sphere {
            center: Vector3::new(0.0, -1000.0, 0.0),
            radius: 1000.0,
            material: MaterialEnum::from(Lambertian {
                albedo: LinSrgb::new(0.5, 0.5, 0.5),
            }),
        },
        Sphere {
            center: Vector3::new(0.0, 1.0, 0.0),
            radius: 1.0,
            material: MaterialEnum::from(Dielectric { index: 1.5 }),
        },
        Sphere {
            center: Vector3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
            material: MaterialEnum::from(Lambertian {
                albedo: LinSrgb::new(0.4, 0.2, 0.1),
            }),
        },
        Sphere {
            center: Vector3::new(4.0, 1.0, 0.0),
            radius: 1.0,
            material: MaterialEnum::from(Metal {
                albedo: LinSrgb::new(0.7, 0.6, 0.5),
                fuzz: 0.0,
            }),
        },
    ];
    HittableList {
        hittables: spheres
            .chain(others.into_iter().map(|s| s.into()))
            .collect(),
    }
}

pub fn render() -> Vec<u32> {
    let scene = construct_scene(&mut rand::thread_rng());

    let origin = Vector3::new(13.0, 2.0, 3.0);
    let camera = Camera::new(
        origin,
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        20.0f32.to_radians(),
        (WIDTH as f32) / (HEIGHT as f32),
        0.1,
        10.0,
    );

    (0..WIDTH * HEIGHT)
        .into_par_iter()
        .map_init(
            || rand::thread_rng(),
            |mut rng, index| {
                let row = index / WIDTH;
                let col = index % WIDTH;
                // Sample SAMPLE_COUNT times per pixel, then average them.
                let color: palette::LinSrgb = (0..SAMPLE_COUNT)
                    .map(|_| {
                        let u = (col as f32 + rng.gen::<f32>()) / (WIDTH as f32);
                        let v = ((HEIGHT - 1 - row) as f32 + rng.gen::<f32>()) / (HEIGHT as f32);
                        let ray = camera.ray(u, v, &mut rng);
                        color(&ray, &scene, 0, &mut rng)
                    })
                    .fold(LinSrgb::new(0.0, 0.0, 0.0), |a, b| a + b)
                    / (SAMPLE_COUNT as f32);

                let color = Srgb::from_linear(color).into_format::<u8>();
                (color.red as u32) << 16 | (color.green as u32) << 8 | (color.blue as u32) << 0
            },
        )
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::new("lufasu", WIDTH, HEIGHT, WindowOptions::default())?;

    let now = std::time::Instant::now();
    let buffer = render();
    let elapsed = now.elapsed();
    window.update_with_buffer(&buffer)?;
    println!(
        "Rendered in {:?} ({:?} per pixel, {:?} per ray)",
        elapsed,
        elapsed / ((WIDTH * HEIGHT) as u32),
        elapsed / ((WIDTH * HEIGHT * SAMPLE_COUNT) as u32)
    );

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update();
    }
    Ok(())
}
