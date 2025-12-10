use image::{ImageBuffer, Rgb};
use naturalneighbor::{Interpolator, Point};
fn main() {
    let (img_w, img_h) = (800, 800);

    let mut img = ImageBuffer::from_pixel(img_w, img_h, Rgb([255 as u8, 255, 255]));

    let mut points = Vec::new();
    let mut weights = Vec::new();
    let grid_size = 20;
    for i in 0..=grid_size {
        for j in 0..=grid_size {
            let x = i as f64 * (img_w as f64 / grid_size as f64);
            let y = j as f64 * (img_h as f64 / grid_size as f64);
            points.push(Point { x, y });
            let weight = ((i + j) % 2) as f64 / 1.0; // alternating weights of 0.0 and 1.0
            weights.push(weight);
        }
    }

    // Create an interpolator
    let interpolator = Interpolator::new(&points);

    // Draw the interpolated colors on the image
    for x in 0..img_w {
        for y in 0..img_h {
            let v = interpolator
                .interpolate(
                    &weights,
                    Point {
                        x: x as f64,
                        y: y as f64,
                    },
                )
                .unwrap();

            if let Some(v) = v {
                img.put_pixel(
                    x as u32,
                    y as u32,
                    Rgb([(v * 255.0) as u8, (v * 255.0) as u8, (v * 255.0) as u8]),
                );
            }
        }
    }

    // Save the image as a PNG file
    img.save("grid.png").unwrap();
}
