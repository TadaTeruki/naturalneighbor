use image::{ImageBuffer, Rgb};
use naturalneighbor::{InterpolatorBuilder, Point};

fn main() {
    let img_w = 500;
    let img_h = 500;

    // Create a new 500x500 white image
    let mut img = ImageBuffer::from_pixel(img_w, img_h, Rgb([255 as u8, 255, 255]));

    
    let points: Vec<Point> = (0..100)
        .map(|_| Point {
            x: rand::random::<f64>() * img_w as f64,
            y: rand::random::<f64>() * img_h as f64,
        })
        .collect();

    let color: Vec<Rgb<u8>> = (0..100)
        .map(|_| {
            Rgb([
                rand::random::<u8>() % 2 * 255,
                rand::random::<u8>() % 2 * 255,
                rand::random::<u8>() % 2 * 255,
            ])
        })
        .collect();

    let interpolator = InterpolatorBuilder::new()
        .set_points(&points)
        .set_items(&color)
        .build()
        .unwrap();

    // Draw points as black circles on the image
    let radius = 3.0;
    points.iter().enumerate().for_each(|(i, site)| {
        for x in ((site.x - radius) as i32)..((site.x + radius) as i32) {
            for y in ((site.y - radius) as i32)..((site.y + radius) as i32) {
                if x < 0 || x >= img_w as i32 || y < 0 || y >= img_h as i32 {
                    continue;
                }
                img.put_pixel(x as u32, y as u32, color[i]);
            }
        }
    });

    for x in 0..img_w {
        for y in 0..img_h {
            let intp = interpolator.interpolate(Point {
                x: x as f64,
                y: y as f64,
            }, |a, b| { //add
                Rgb([
                    (a.0[0] as u16 + b.0[0] as u16).min(255) as u8,
                    (a.0[1] as u16 + b.0[1] as u16).min(255) as u8,
                    (a.0[2] as u16 + b.0[2] as u16).min(255) as u8,
                ])
            },|a, weight| {
                Rgb([
                    (a.0[0] as f64 * weight) as u8,
                    (a.0[1] as f64 * weight) as u8,
                    (a.0[2] as f64 * weight) as u8,
                ])
            });
            //img.put_pixel(x as u32, y as u32, color[i]);

            if let Some(color) = intp {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
    /*
    let points: Vec<Point> = (0..5000)
    .map(|_| Point {
        x: rand::random::<f64>() * img_w as f64,
        y: rand::random::<f64>() * img_h as f64,
    })
    .collect();

    let radius = 3.0;
    points.iter().enumerate().for_each(|(i, site)| {
        for x in ((site.x - radius) as i32)..((site.x + radius) as i32) {
            for y in ((site.y - radius) as i32)..((site.y + radius) as i32) {
                if x < 0 || x >= img_w as i32 || y < 0 || y >= img_h as i32 {
                    continue;
                }

                let intp = interpolator.interpolate(Point {
                    x: site.x as f64,
                    y: site.y as f64,
                }, |a, b| { //add
                    Rgb([
                        (a.0[0] as u16 + b.0[0] as u16).min(255) as u8,
                        (a.0[1] as u16 + b.0[1] as u16).min(255) as u8,
                        (a.0[2] as u16 + b.0[2] as u16).min(255) as u8,
                    ])
                },|a, weight| {
                    Rgb([
                        (a.0[0] as f64 * weight) as u8,
                        (a.0[1] as f64 * weight) as u8,
                        (a.0[2] as f64 * weight) as u8,
                    ])
                });
                //img.put_pixel(x as u32, y as u32, color[i]);

                if let Some(color) = intp {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    });
    */

    /*
    let ln = intp.clone().unwrap().len();

    intp.unwrap().iter().enumerate().for_each(|(i,v)| {
        let site = &points[*v];
        let weight = i as f64 / ln as f64;
        // draw site 
        for x in ((site.x - radius*2.) as i32)..((site.x + radius*2.) as i32) {
            for y in ((site.y - radius*2.) as i32)..((site.y + radius*2.) as i32) {
                if x < 0 || x >= img_w as i32 || y < 0 || y >= img_h as i32 {
                    continue;
                }
                
                img.put_pixel(x as u32, y as u32, Rgb([
                    (100.0 * (1.0-weight)) as u8,
                    (100.0 * (1.0-weight)) as u8,
                    (255.0 * (1.0-weight)) as u8,
                ]));
            }
        }
    });
    */


    // Save the image as a PNG file
    img.save("points.png").unwrap();
}
