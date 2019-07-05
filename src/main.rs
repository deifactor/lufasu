mod geometry;

use minifb::{Window, WindowOptions};
use nalgebra::Vector3;
use palette::{LinSrgb, Mix};
use rand::prelude::*;
use rayon::prelude::*;
use std::error::Error;
use std::sync::{Arc, Mutex};

use geometry::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 400;
// Number of per-pixel samples.
const SAMPLE_COUNT: usize = 100;

pub fn render_into(buf: &mut [u32]) {
    let center_sphere = Sphere {
        center: Vector3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    };
    let floor = Sphere {
        center: Vector3::new(0.0, -100.5, -1.0),
        radius: 100.0,
    };
    let scene = HittableList {
        hittables: vec![Box::new(center_sphere), Box::new(floor)],
    };
    let camera = Camera {
        origin: Vector3::new(0.0, 0.0, 0.0),
        lower_left: Vector3::new(-2.0, -1.0, -1.0),
        horizontal: Vector3::new(4.0, 0.0, 0.0),
        vertical: Vector3::new(0.0, 2.0, 0.0),
    };

    // Since no worker thread will ever write to the same part of the buffer as
    // another, in *theory* we could just share it directly... but there may be
    // other issues with that, and in practice just locking to write into it
    // should give us a speedup.
    let buf_mutex = Arc::new(Mutex::new(buf));
    // Render each column in parallel to avoid locking the buffer for each pixel.
    (0..HEIGHT)
        .into_par_iter()
        .for_each_with(buf_mutex, |buf_mutex, row| {
            let mut rng = rand::thread_rng();
            let mut temp = vec![0u32; WIDTH];
            for col in 0..WIDTH {
                // Sample SAMPLE_COUNT times per pixel, then average them.
                let color: palette::LinSrgb = (0..SAMPLE_COUNT)
                    .map(|_| {
                        let u = (col as f32 + rng.gen::<f32>()) / (WIDTH as f32);
                        let v = ((HEIGHT - 1 - row) as f32 + rng.gen::<f32>()) / (HEIGHT as f32);
                        let ray = camera.ray(u, v);
                        if let Some(hit_record) = scene.hits(&ray, 0.0001, std::f32::INFINITY) {
                            let n =
                                (hit_record.normal.unwrap() + Vector3::new(1.0, 1.0, 1.0)) / 2.0;
                            LinSrgb::new(n.x, n.y, n.z)
                        } else {
                            let t = (ray.direction().y + 1.0) / 2.0;
                            let white = LinSrgb::new(1.0, 1.0, 1.0);
                            let blue = LinSrgb::new(0.0, 0.0, 1.0);
                            white.mix(&blue, t as f32)
                        }
                    })
                    .fold(LinSrgb::new(0.0, 0.0, 0.0), |a, b| a + b)
                    / (SAMPLE_COUNT as f32);

                let color = color.into_format::<u8>();
                temp[col] =
                    (color.red as u32) << 16 | (color.green as u32) << 8 | (color.blue as u32) << 0;
            }
            let mut buf = buf_mutex.lock().unwrap();
            buf[row * WIDTH..row * WIDTH + WIDTH].copy_from_slice(&temp);
        });
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut window = Window::new("lufasu", WIDTH, HEIGHT, WindowOptions::default())?;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let now = std::time::Instant::now();
    render_into(&mut buffer);
    let elapsed = now.elapsed();
    window.update_with_buffer(&buffer)?;
    println!(
        "Rendered in {:?} ({:?} per pixel)",
        elapsed,
        elapsed / ((WIDTH * HEIGHT) as u32)
    );

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update();
    }
    Ok(())
}
