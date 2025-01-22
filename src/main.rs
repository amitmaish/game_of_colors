use image;
use image::Rgb;
use rand;
use std::env;
use std::io::{Cursor, Read};
use std::path::Path;

#[derive(Debug)]
enum Input<'a> {
    Pipe,
    Path(Option<&'a Path>),
}

struct Configuration<'a> {
    imgx: u32,
    imgy: u32,

    generations: u32,

    clamp_min: f32,
    clamp_max: f32,
    threshold: f32,

    input: Input<'a>,
    output: String,
}

const BLACK: Rgb<f32> = Rgb::<f32>([0.0; 3]);

trait Pixel {
    fn new(r: f32, g: f32, b: f32) -> Self;
    fn length(&self) -> f32;
    fn threshold(&self, threshold: f32) -> Self;
    fn clamp(&self, min: f32, max: f32) -> Self;
    fn normalize(&self) -> Self;
    fn dot(&self, v: &Self) -> f32;
    fn rand() -> Self;
}

impl Pixel for Rgb<f32> {
    fn new(r: f32, g: f32, b: f32) -> Self {
        Rgb::<f32>([r, g, b])
    }

    fn threshold(&self, threshold: f32) -> image::Rgb<f32> {
        if self.length() >= threshold {
            self.clone()
        } else {
            BLACK
        }
    }

    fn length(&self) -> f32 {
        let squared_length = (self[0] * self[0]) + (self[1] * self[1]) + (self[2] * self[2]);
        squared_length.sqrt()
    }

    fn clamp(&self, min: f32, max: f32) -> image::Rgb<f32> {
        Rgb::<f32>([
            self[0].clamp(min, max),
            self[1].clamp(min, max),
            self[2].clamp(min, max),
        ])
    }

    fn normalize(&self) -> Self {
        match self.length() {
            0.0 => BLACK,
            _ => image::Rgb::<f32>([
                self[0] / self.length(),
                self[0] / self.length(),
                self[0] / self.length(),
            ]),
        }
    }

    fn dot(&self, v: &Self) -> f32 {
        let u = self;
        (u[0] * v[0]) + (u[1] * v[1]) + (u[2] * v[2])
    }

    fn rand() -> Self {
        image::Rgb([
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        ])
    }
}

fn add_pixel(lhs: &mut Rgb<f32>, rhs: Rgb<f32>) {
    lhs[0] += rhs[0];
    lhs[1] += rhs[1];
    lhs[2] += rhs[2];
}

#[derive(PartialEq, Debug)]
struct CellState {
    alive: bool,
    neighborhood: f32,
    neighborhood_color: Rgb<f32>,
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
    eprintln!("\ngame_of_colors\n");

    let mut config = Configuration {
        imgx: 100,
        imgy: 100,
        generations: 100,
        input: Input::Path(None),
        clamp_min: 0.0,
        clamp_max: 1.0,
        threshold: 0.0,
        output: String::from("output/"),
    }; // Set the default configuration

    let mut args = env::args();

    let _ = args.next(); // ignore first item

    let mut input_path_arg: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.trim() {
            "-i" => {
                input_path_arg = Some(String::from(args.next().unwrap().trim()));
            }
            "-o" => {
                config.output = String::from(args.next().unwrap().trim());
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
            "-clamp_min" => {
                config.clamp_min = String::from(args.next().unwrap().trim()).parse().unwrap();
            }
            "-clamp_max" => {
                config.clamp_max = String::from(args.next().unwrap().trim()).parse().unwrap();
            }
            "-threshold" => {
                config.threshold = String::from(args.next().unwrap().trim()).parse().unwrap();
            }
            "-pipe" => {
                config.input = Input::Pipe;
            }

            _ => panic!("Couldn't parse input"),
        }
    }

    let input_path;
    match input_path_arg {
        None => (),
        Some(path) => {
            input_path = path;
            config.input = Input::Path(Some(Path::new(&input_path)));
        }
    }

    let mut imgbuf = match config.input {
        Input::Pipe => {
            let mut buf: Vec<u8> = Vec::new();
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();

            match handle.read_to_end(&mut buf) {
                Result::Err(_x) => (),
                Result::Ok(x) => {
                    eprintln!("read {} bytes", x)
                }
            };

            let buf = Cursor::new(buf);

            let imgbuf = image::ImageReader::new(buf)
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .to_rgb32f();
            let imgbuf = image::DynamicImage::from(imgbuf);

            eprintln!("image dimensions {:?}", (imgbuf.width(), imgbuf.height()));
            config.imgx = imgbuf.width();
            config.imgy = imgbuf.height();

            imgbuf
        }
        Input::Path(None) => generate_random_gen(&config),
        Input::Path(Some(path)) => image::ImageReader::open(path).unwrap().decode().unwrap(),
    };

    for (_x, _y, pixel) in imgbuf.as_mut_rgb32f().expect("all internal images are rgb32f").enumerate_pixels_mut() {
        pixel
            .threshold(config.threshold)
            .clamp(config.clamp_min, config.clamp_max);
    }

    // write inital generation
    imgbuf
        .to_rgb8()
        .save(format!("{}0000.png", config.output))
        .unwrap();

    simulate_life(imgbuf, &config);
}

fn generate_random_gen(config: &Configuration) -> image::DynamicImage {
    let mut imgbuf = image::DynamicImage::new_rgb32f(config.imgx, config.imgy);

    for (_, _, pixel) in imgbuf.as_mut_rgb32f().unwrap().enumerate_pixels_mut() {
        if rand::random::<bool>() {
            *pixel = Rgb::<f32>::rand()
        }
    }

    imgbuf
}

fn simulate_life(imgbuf: image::DynamicImage, config: &Configuration) {
    let mut lastgen = imgbuf;

    for i in 1..config.generations {
        eprintln!("simulating gen {}", i);
        let mut genbuf = image::DynamicImage::new_rgb32f(config.imgx, config.imgy);

        // insert game of life logic here
        for (x, y, pixel) in genbuf.as_mut_rgb32f().unwrap().enumerate_pixels_mut() {
            let cell_state = gather_cell_state(pixel, &lastgen, x, y);

            match cell_state.alive {
                true => {
                    if (cell_state.neighborhood >= 2.0) && (cell_state.neighborhood <= 3.0) {
                        *pixel = *lastgen.as_rgb32f().unwrap().get_pixel(x, y);
                    }
                }
                false => {
                    if cell_state.neighborhood == 3.0 {
                        *pixel = cell_state
                            .neighborhood_color
                            .clamp(config.clamp_min, config.clamp_max);
                    }
                }
            }
        }

        genbuf
            .to_rgb8()
            .save(format!("{}{:04}.png", config.output, i))
            .unwrap();

        lastgen = genbuf;
    }
}

fn gather_cell_state(
    current_pixel: &Rgb<f32>,
    lastgen: &image::DynamicImage,
    x: u32,
    y: u32,
) -> CellState {
    let mut cell_state = CellState::new();

    // check living status
    match lastgen.as_rgb32f().unwrap().get_pixel_checked(x, y) {
        None => (),
        Some(pixel) => {
            if pixel.length() > 0.25 {
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
                .as_rgb32f()
                .unwrap()
                .get_pixel_checked((x as i32 + x_offset) as u32, (y as i32 + y_offset) as u32)
            {
                None => (),
                Some(neighbor_pixel) => {
                    let neighbor = neighbor_pixel;
                    let neighbor_similarity = match current_pixel.normalize() {
                        BLACK => {
                            if *neighbor == BLACK {
                                0.0
                            } else {
                                1.0
                            }
                        }
                        _ => current_pixel.normalize().dot(&neighbor.normalize()),
                    };
                    cell_state.neighborhood += neighbor_similarity;
                    if neighbor_similarity >= 0.25 {
                        add_pixel(&mut cell_state.neighborhood_color, *neighbor);
                    }
                }
            }
        }
    }

    cell_state.neighborhood = (cell_state.neighborhood * 10000.0).round() / 10000.0;
    cell_state.neighborhood_color = Rgb::<f32>([
        cell_state.neighborhood_color[0] / cell_state.neighborhood,
        cell_state.neighborhood_color[1] / cell_state.neighborhood,
        cell_state.neighborhood_color[2] / cell_state.neighborhood,
    ]);

    cell_state
}
