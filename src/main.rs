mod geometry;

use minifb::{Window, WindowOptions};
use nalgebra::Vector3;
use palette::{LinSrgb, Mix};
use std::error::Error;

use geometry::*;

const WIDTH: usize = 800;
const HEIGHT: usize = 400;

pub fn render_into(buf: &mut [u32]) {
    let origin = Vector3::new(0.0, 0.0, 0.0);
    let lower_left = Vector3::new(-2.0, -1.0, -1.0);
    let horizontal = Vector3::new(4.0, 0.0, 0.0);
    let vertical = Vector3::new(0.0, 2.0, 0.0);

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
    for col in 0..WIDTH {
        for row in 0..HEIGHT {
            let u = (col as f32) / (WIDTH as f32);
            let v = ((HEIGHT - 1 - row) as f32) / (HEIGHT as f32);
            let ray = Ray::new(origin, lower_left + u * horizontal + v * vertical);

            let color = if let Some(hit_record) = scene.hits(&ray, 0.0001, std::f32::INFINITY) {
                let n = (hit_record.normal.unwrap() + Vector3::new(1.0, 1.0, 1.0)) / 2.0;
                LinSrgb::new(n.x, n.y, n.z)
            } else {
                let t = (ray.direction().y + 1.0) / 2.0;
                let white = LinSrgb::new(1.0, 1.0, 1.0);
                let blue = LinSrgb::new(0.0, 0.0, 1.0);
                white.mix(&blue, t as f32)
            };
            let color = color.into_format::<u8>();
            buf[row * WIDTH + col] =
                (color.red as u32) << 16 | (color.green as u32) << 8 | (color.blue as u32) << 0;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let v: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
    v.normalize();
    let mut window = Window::new("lufasu", WIDTH, HEIGHT, WindowOptions::default())?;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    render_into(&mut buffer);
    window.update_with_buffer(&buffer)?;

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update();
    }
    Ok(())
}
