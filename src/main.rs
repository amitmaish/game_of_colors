use image;
use rand;
use std::env;
use std::ops::AddAssign;
use std::ops::Div;
use std::path::Path;

struct Configuration<'a> {
    imgx: u32,
    imgy: u32,

    generations: u32,

    input_path: Option<&'a Path>,
    output_path: String,
}

#[derive(PartialEq, Debug)]
struct Pixel {
    r: f64,
    g: f64,
    b: f64,
}

impl Pixel {
    fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    fn from_rgb8(pixel: &image::Rgb<u8>) -> Self {
        Self {
            r: pixel[0] as f64 / u8::MAX as f64,
            g: pixel[1] as f64 / u8::MAX as f64,
            b: pixel[2] as f64 / u8::MAX as f64,
        }
    }

    fn as_rgb8(&self) -> image::Rgb<u8> {
        image::Rgb([
            (self.r * (u8::MAX as f64)) as u8,
            (self.g * (u8::MAX as f64)) as u8,
            (self.b * (u8::MAX as f64)) as u8,
        ])
    }

    fn length(&self) -> f64 {
        let squared_length = (self.r * self.r) + (self.g * self.g) + (self.b * self.b);
        squared_length.sqrt()
    }

    fn normalize(&self) -> Pixel {
        match self.length() {
            0.0 => Pixel::BLACK,
            _ => Pixel {
                r: self.r / self.length(),
                g: self.g / self.length(),
                b: self.b / self.length(),
            },
        }
    }

    fn dot(&self, v: &Pixel) -> f64 {
        let u = self;
        (u.r * v.r) + (u.g * v.g) + (u.b * v.b)
    }

    fn rand() -> Pixel {
        Pixel {
            r: rand::random::<f64>(),
            g: rand::random::<f64>(),
            b: rand::random::<f64>(),
        }
    }

    const WHITE: Pixel = Pixel {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    const _RED: Pixel = Pixel {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };

    const _GREEN: Pixel = Pixel {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };

    const _BLUE: Pixel = Pixel {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };

    const BLACK: Pixel = Pixel {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
}

impl AddAssign for Pixel {
    fn add_assign(&mut self, other: Pixel) {
        *self = Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl Div<f64> for Pixel {
    type Output = Self;

    fn div(self, x: f64) -> Self::Output {
        Self {
            r: self.r / x,
            g: self.g / x,
            b: self.b / x,
        }
    }
}

#[derive(PartialEq, Debug)]
struct CellState {
    alive: bool,
    neighborhood: f64,
    neighborhood_color: Pixel,
}

impl CellState {
    fn new() -> Self {
        Self {
            alive: false,
            neighborhood: 0.0,
            neighborhood_color: Pixel::new(0.0, 0.0, 0.0),
        }
    }
}

fn main() {
    let mut config = Configuration {
        imgx: 100,
        imgy: 100,
        generations: 100,
        input_path: None,
        output_path: String::from("output/"),
    }; // Set the default configuration

    let mut args = env::args();

    let _ = args.next(); // ignore first item

    let mut input_path_arg: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.trim() {
            "-i" => {
                input_path_arg = Some(args.next().unwrap());
            }
            "-o" => {
                config.output_path = String::from(args.next().unwrap().trim());
            }
            "-x" => {
                config.imgx = String::from(args.next().unwrap().trim()).parse().unwrap();
            }
            "-y" => {
                config.imgy = String::from(args.next().unwrap().trim()).parse().unwrap();
            }
            "-g" => {
                config.generations = String::from(args.next().unwrap().trim()).parse().unwrap();
            }
            _ => panic!("Couldn't parse input"),
        }
    }

    let input_path;

    match input_path_arg {
        None => config.input_path = None,
        Some(path) => {
            input_path = path;
            config.input_path = Some(Path::new(&input_path));
        }
    }

    let imgbuf = match config.input_path {
        None => generate_random_gen(&config),
        Some(input_path) => {
            let buf = image::ImageReader::open(input_path)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();

            config.imgx = buf.width();
            config.imgy = buf.height();

            buf
        }
    };

    // write inital generation
    imgbuf
        .save(format!("{}0000.png", config.output_path))
        .unwrap();

    simulate_life(imgbuf, &config);
}

fn generate_random_gen(config: &Configuration) -> image::RgbImage {
    let mut imgbuf = image::RgbImage::new(config.imgx, config.imgy);

    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        match rand::random() {
            true => *pixel = Pixel::rand().as_rgb8(),
            false => (),
        }
    }

    imgbuf
}

fn simulate_life(imgbuf: image::RgbImage, config: &Configuration) {
    let mut lastgen = imgbuf;

    for i in 1..config.generations {
        let mut genbuf = image::RgbImage::new(config.imgx, config.imgy);

        // insert game of life logic here
        for (x, y, pixel) in genbuf.enumerate_pixels_mut() {
            let cell_state = gather_cell_state(&Pixel::from_rgb8(pixel), &lastgen, x, y);

            match cell_state.alive {
                true => {
                    if (cell_state.neighborhood >= 2.0) && (cell_state.neighborhood <= 3.0) {
                        *pixel = Pixel::from_rgb8(lastgen.get_pixel(x, y)).as_rgb8();
                    }
                }
                false => {
                    if cell_state.neighborhood == 3.0 {
                        *pixel = cell_state.neighborhood_color.as_rgb8();
                    }
                }
            }
        }

        genbuf
            .save(format!("{}{:04}.png", config.output_path, i))
            .unwrap();

        lastgen = genbuf;
    }
}

fn gather_cell_state(
    current_pixel: &Pixel,
    lastgen: &image::RgbImage,
    x: u32,
    y: u32,
) -> CellState {
    let mut cell_state = CellState::new();

    // check living status
    match lastgen.get_pixel_checked(x, y) {
        None => (),
        Some(pixel) => {
            if Pixel::from_rgb8(pixel).length() > 0.25 {
                cell_state.alive = true
            }
        }
    }

    // itterate over neighbors
    for x_offset in -1i32..=1 {
        for y_offset in -1i32..=1 {
            if (x_offset, y_offset) == (0, 0) {
                continue;
            }
            match lastgen
                .get_pixel_checked((x as i32 + x_offset) as u32, (y as i32 + y_offset) as u32)
            {
                None => (),
                Some(neighbor_pixel) => {
                    let neighbor = Pixel::from_rgb8(neighbor_pixel);
                    let neighbor_similarity = match current_pixel.normalize() {
                        Pixel::BLACK => Pixel::WHITE.normalize().dot(&neighbor.normalize()),
                        _ => current_pixel.normalize().dot(&neighbor.normalize()),
                    };
                    cell_state.neighborhood += neighbor_similarity;
                    if neighbor_similarity >= 0.25 {
                        cell_state.neighborhood_color += neighbor;
                    }
                }
            }
        }
    }

    cell_state.neighborhood = (cell_state.neighborhood * 10000.0).round() / 10000.0;
    cell_state.neighborhood_color = cell_state.neighborhood_color / cell_state.neighborhood;

    cell_state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Configuration<'static> {
        Configuration {
            imgx: 3,
            imgy: 3,

            generations: 3,

            input_path: None,
            output_path: "output/".to_string(),
        }
    }

    fn empty_buf() -> image::RgbImage {
        let config = test_config();
        image::RgbImage::new(config.imgx, config.imgy)
    }

    fn buf1() -> image::RgbImage {
        let mut buf = empty_buf();

        buf.put_pixel(0, 0, Pixel::WHITE.as_rgb8());
        buf.put_pixel(1, 0, Pixel::WHITE.as_rgb8());
        buf.put_pixel(2, 2, Pixel::WHITE.as_rgb8());

        buf
    }

    #[test]
    fn normalize() {
        assert_eq!(Pixel::_RED.normalize().length(), 1.0);
        assert_eq!(Pixel::WHITE.normalize().length(), 1.0);
    }

    #[test]
    fn dot_product() {
        assert_eq!(Pixel::_RED.normalize().dot(&Pixel::_RED.normalize()), 1.0);

        assert_eq!(Pixel::_RED.normalize().dot(&Pixel::_BLUE.normalize()), 0.0);

        let close_enough =
            (Pixel::WHITE.normalize().dot(&Pixel::WHITE.normalize()) - 1.0).abs() <= 0.0001;
        assert_eq!(close_enough, true);
    }

    #[test]
    fn cell_state_test() {
        let buf1 = buf1();

        let state1 = gather_cell_state(&Pixel::from_rgb8(buf1.get_pixel(1, 1)), &buf1, 1, 1);

        assert_eq!(
            state1,
            CellState {
                alive: false,
                neighborhood: 3.0,
                neighborhood_color: Pixel::WHITE,
            }
        );
    }
}
