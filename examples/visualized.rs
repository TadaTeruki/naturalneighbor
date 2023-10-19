use image::{ImageBuffer, Rgb};
use naturalneighbor::{InterpolatorBuilder, Point};
use rand::Rng;

fn main() {
    let img_w = 500;
    let img_h = 500;

    // Create a new 500x500 white image
    let mut img = ImageBuffer::from_pixel(img_w, img_h, Rgb([255 as u8, 255, 255]));

    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([4; 32]);
    
    let points: Vec<Point> = (0..100)
        .map(|_| Point {
            x: rng.gen::<f64>() * img_w as f64,
            y: rng.gen::<f64>() * img_h as f64,
        })
        .collect();

    let color = (0..100)
        .map(|_| {
            [
                (rng.gen::<u8>() % 2) as f64 * 255.,
                (rng.gen::<u8>() % 2) as f64 * 255.,
                (rng.gen::<u8>() % 2) as f64 * 255.,
            ]
        })
        .collect::<Vec<_>>();

    let interpolator = InterpolatorBuilder::new()
        .set_points(&points)
        .set_items(&color)
        .build()
        .unwrap();
    
    for x in img_w/8*2..img_w/8*6 {
        for y in img_h/8*2..img_h/8*6 {
            let intp = interpolator.interpolate(Point {
                x: x as f64,
                y: y as f64,
            }, |a, b| { //add
                [
                    a[0] + b[0],
                    a[1] + b[1],
                    a[2] + b[2],
                ]
            },|a, weight| {
                [
                    a[0] * weight,
                    a[1] * weight,
                    a[2] * weight,
                ]
            });
            //img.put_pixel(x as u32, y as u32, color[i]);

            if let Some(c) = intp {
                img.put_pixel(x as u32, y as u32, Rgb([
                    c[0] as u8,
                    c[1] as u8,
                    c[2] as u8,
                ]));
            }
        }
    }

    let radius = 3.0;

    let search_site = Point {
        x: 250.0,
        y: 250.0,
    };

    for x in ((search_site.x - radius) as i32)..((search_site.x + radius) as i32) {
        for y in ((search_site.y - radius) as i32)..((search_site.y + radius) as i32) {
            if x < 0 || x >= img_w as i32 || y < 0 || y >= img_h as i32 {
                continue;
            }
            
            img.put_pixel(x as u32, y as u32, Rgb([255,0,0]));
        }
    }

    // Draw points as black circles on the image
    
    points.iter().enumerate().for_each(|(i, site)| {
        for x in ((site.x - radius) as i32)..((site.x + radius) as i32) {
            for y in ((site.y - radius) as i32)..((site.y + radius) as i32) {
                if x < 0 || x >= img_w as i32 || y < 0 || y >= img_h as i32 {
                    continue;
                }
                // if x and y is on border, draw black
                if x == (site.x - radius) as i32 || x == (site.x + radius) as i32 - 1 || y == (site.y - radius) as i32 || y == (site.y + radius) as i32 - 1 {
                    img.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
                    continue;
                }

                img.put_pixel(x as u32, y as u32, Rgb([
                    color[i][0] as u8,
                    color[i][1] as u8,
                    color[i][2] as u8,
                ]));
            }
        }
    });


    // Save the image as a PNG file
    img.save("points.png").unwrap();
}
