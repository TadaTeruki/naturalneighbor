use image::{ImageBuffer, Rgb};
use naturalneighbor::{InterpolatorBuilder, Point};
fn main() {
    let (img_w, img_h) = (800, 800);

    let mut img = ImageBuffer::from_pixel(img_w, img_h, Rgb([255 as u8, 255, 255]));

    let points = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 800.0, y: 0.0 },
        Point { x: 800.0, y: 800.0 },
        Point { x: 0.0, y: 800.0 },
        Point { x: 454.0, y: 223.0 },
        Point { x: 302.0, y: 345.0 },
        Point { x: 258.0, y: 632.0 },
        Point { x: 620.0, y: 513.0 },
        Point { x: 285.0, y: 479.0 },
        Point { x: 444.0, y: 453.0 },
        Point { x: 537.0, y: 315.0 },
        Point { x: 425.0, y: 610.0 },
        Point { x: 528.0, y: 223.0 },
    ];

    // weights of the points to be interpolated
    let weights = [
        0.4, 0.9, 0.0, 0.7, 0.1, 0.3, 0.2, 0.4, 0.9, 0.0, 0.7, 0.8, 0.5,
    ];

    // Create an interpolator
    let interpolator = InterpolatorBuilder::default()
        .set_points(&points)
        .set_values(&weights)
        .build()
        .unwrap();

    // Draw the interpolated colors on the image
    for x in 0..img_w {
        for y in 0..img_h {
            let v = interpolator.interpolate(Point {
                x: x as f64,
                y: y as f64,
            });

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
    img.save("minimal.png").unwrap();
}
