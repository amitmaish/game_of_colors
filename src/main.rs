use image;

fn main() {
    let imgx = 5;
    let imgy = 5;

    let generations = 5;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::RgbImage::new(imgx, imgy);

    // create initial generation as a blinker and save it
    for x in 1..=3 {
        imgbuf.put_pixel(x, 2, image::Rgb([255, 255, 255]))
    }
    imgbuf.save("output/0001.png").unwrap();

    let mut lastgen = imgbuf;

    for i in 0..generations {
        let mut genbuf = image::RgbImage::new(imgx, imgy);

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

        let _ = genbuf.save(format!("output/{:04}.png", i));

        lastgen = genbuf;
    }
}
