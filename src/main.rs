use std::env;
use std::path::Path;
use rand::Rng;
use image;

struct Configuration<'a> {
    imgx: u32,
    imgy: u32,

    generations: u32,

    input: 'a Option<Path>,
    output: String,
}

fn main() {
    let mut config = Configuration {
        imgx: 100,
        imgy: 100,
        generations: 100,
        input: None,
        output: String::from("output/"),
    };


    let mut args = env::args();

    let _ = args.next(); // ignore first item

    while let Some(arg) = args.next() {
        match arg.trim() {
        "-i" => config.input = Some(Path::new(&String::from(args.next().unwrap().trim()))),
        "-o" => {config.output = String::from(args.next().unwrap().trim()); println!("output: {}", config.output);},
        _ => panic!("Couldn't parse input"),
        }
    }
    
    let mut imgbuf = match config.input {
        None => generate_random_gen(&config),
        Some(input) => image::ImageReader::open(config.input).unwrap().decode().unwrap().into_rgb8(),
    };
    
    // write inital generation
    imgbuf.save(format!("{}0000.png", config.output)).unwrap();

    simulate_life(imgbuf, &config);
}

//fn read_input_gen(config: &Configuration) -> image::RgbImage {

//}

fn generate_random_gen(config: &Configuration) -> image::RgbImage {
    let mut imgbuf = image::RgbImage::new(config.imgx, config.imgy);
    
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        let random_number = rand::thread_rng().gen_range(0..=1);
        if random_number == 1 { *pixel = image::Rgb([255, 255, 255]); }
    }

    imgbuf
}

fn simulate_life(imgbuf: image::RgbImage, config: &Configuration) {
    let mut lastgen = imgbuf;

    for i in 1..config.generations {
        let mut genbuf = image::RgbImage::new(config.imgx, config.imgy);

        // insert game of life logic here
        for (x, y, pixel) in genbuf.enumerate_pixels_mut() {
            let mut live_neighbors = 0;
            let mut pixel_alive = false;
            // itterate over neighbors
            for x_offset in -1i32..=1 {
                for y_offset in -1i32..=1 {
                    match lastgen.get_pixel_checked(
                        (x as i32 + x_offset) as u32,
                        (y as i32 + y_offset) as u32,
                    ) {
                        None => (),
                        Some(color) => {
                            if color[0] > 0 {
                                match (x_offset, y_offset) {
                                    (0, 0) => pixel_alive = true,
                                    _ => live_neighbors += 1,
                                }
                            }
                        }
                    }
                }
            }

            match pixel_alive {
                true => {
                    if (live_neighbors >= 2) && (live_neighbors <= 3) {
                        *pixel = image::Rgb([255, 255, 255]);
                    }
                }
                false => {
                    if live_neighbors == 3 {
                        *pixel = image::Rgb([255, 255, 255]);
                    }
                }
            }
        }

        genbuf.save(format!("{}{:04}.png", config.output, i)).unwrap();

        lastgen = genbuf;
    }
}
