use image::{ImageBuffer, Rgb};
use naturalneighbor::{InterpolatorBuilder, Lerpable, Point};
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    fn to_rgb(&self) -> Rgb<u8> {
        Rgb([
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
        ])
    }
}

impl Lerpable for Color {
    fn lerp(&self, other: &Self, weight: f64) -> Self {
        Self {
            r: self.r * (1.0 - weight) + other.r * weight,
            g: self.g * (1.0 - weight) + other.g * weight,
            b: self.b * (1.0 - weight) + other.b * weight,
        }
    }
}

static PALLETE: [Color; 7] = [
    Color {
        r: 1.0,
        g: 0.3,
        b: 0.3,
    },
    Color {
        r: 0.3,
        g: 1.0,
        b: 0.3,
    },
    Color {
        r: 0.3,
        g: 0.3,
        b: 1.0,
    },
    Color {
        r: 1.0,
        g: 1.0,
        b: 0.3,
    },
    Color {
        r: 1.0,
        g: 0.3,
        b: 1.0,
    },
    Color {
        r: 0.3,
        g: 1.0,
        b: 1.0,
    },
    Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    },
];

fn main() {
    let (img_w, img_h) = (1000, 500);
    let n = 100;
    let radius = 3.0;

    let mut img = ImageBuffer::from_pixel(img_w, img_h, Rgb([255 as u8, 255, 255]));
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);

    let points: Vec<Point> = (0..n)
        .map(|_| Point {
            x: rng.gen::<f64>() * img_w as f64,
            y: rng.gen::<f64>() * img_h as f64,
        })
        .collect();

    let colors = (0..n)
        .map(|_| PALLETE[rng.gen::<usize>() % PALLETE.len()])
        .collect::<Vec<_>>();

    let interpolator = InterpolatorBuilder::default()
        .set_points(&points)
        .set_items(&colors)
        .build()
        .unwrap();

    for x in 0..img_w {
        for y in 0..img_h {
            let intp = interpolator.interpolate(Point {
                x: x as f64,
                y: y as f64,
            });

            if let Some(c) = intp {
                img.put_pixel(x as u32, y as u32, c.to_rgb());
            }
        }
    }

    // Draw points as black circles on the image
    points.iter().enumerate().for_each(|(i, site)| {
        for x in ((site.x - radius) as i32)..((site.x + radius) as i32) {
            for y in ((site.y - radius) as i32)..((site.y + radius) as i32) {
                if x < 0 || x >= img_w as i32 || y < 0 || y >= img_h as i32 {
                    continue;
                }
                if x == (site.x - radius) as i32
                    || x == (site.x + radius) as i32 - 1
                    || y == (site.y - radius) as i32
                    || y == (site.y + radius) as i32 - 1
                {
                    img.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
                } else {
                    img.put_pixel(x as u32, y as u32, colors[i].to_rgb());
                }
            }
        }
    });

    // Save the image as a PNG file
    img.save("color.png").unwrap();
}
