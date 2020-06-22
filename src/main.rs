extern crate image;
extern crate itertools;
extern crate num_complex;
use crate::image::GenericImage;
use crate::image::GenericImageView;
use itertools::Itertools;

type Criteria = fn(&image::Rgb<u8>) -> u32;

fn get_red(item: &image::Rgb<u8>) -> u32 {
    item.0[0] as u32
}
fn get_green(item: &image::Rgb<u8>) -> u32 {
    item.0[1] as u32
}
fn get_blue(item: &image::Rgb<u8>) -> u32 {
    item.0[2] as u32
}
fn get_average(item: &image::Rgb<u8>) -> u32 {
    (item.0[0] as u32 + item.0[1] as u32 + item.0[2] as u32) / 3
}
fn get_hue(item: &image::Rgb<u8>) -> u32 {
    let r = item.0[0] as f32 / 255.0;
    let g = item.0[1] as f32 / 255.0;
    let b = item.0[2] as f32 / 255.0;
    let c_max = f32::max(f32::max(r, g), b);
    let c_min = f32::min(f32::min(r, g), b);
    let delta = c_max - c_min;
    if delta == 0.0 {
        return 0;
    }
    if c_max == r {
        return 60 * (((g - b) / delta) as u32 % 6);
    } else if c_max == g {
        return 60 * (((b - r) / delta) as u32 + 2);
    } else if c_max == b {
        return 60 * (((r - g) / delta) as u32 + 4);
    }
    0
}
fn get_saturation(item: &image::Rgb<u8>) -> u32 {
    let r = item.0[0] as f32 / 255.0;
    let g = item.0[1] as f32 / 255.0;
    let b = item.0[2] as f32 / 255.0;
    let c_max = f32::max(f32::max(r, g), b);
    let c_min = f32::min(f32::min(r, g), b);
    let delta = c_max - c_min;
    if delta == 0.0 {
        return 0;
    }
    let l = (c_max + c_min) / 2.0;
    (delta / (1.0 - (2.0 * l - 1.0).abs())) as u32
}
fn get_lightness(item: &image::Rgb<u8>) -> u32 {
    let r = item.0[0] as f32 / 255.0;
    let g = item.0[1] as f32 / 255.0;
    let b = item.0[2] as f32 / 255.0;
    let c_max = f32::max(f32::max(r, g), b);
    let c_min = f32::min(f32::min(r, g), b);
    let delta = c_max - c_min;
    if delta == 0.0 {
        return 0;
    }
    ((c_max + c_min) / 2.0) as u32
}

type Sorter = fn(
    buf: &image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
    crit: Criteria,
) -> image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>;

fn basic_sort(
    buf: &image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
    crit: Criteria,
) -> image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>> {
    let sorted_pixels = buf
        .pixels()
        .sorted_by_key(|p| crit(p))
        .flat_map(|p| p.0.iter())
        .cloned()
        .collect();

    let width = buf.width();
    let height = buf.height();
    image::ImageBuffer::from_vec(width, height, sorted_pixels).unwrap()
}

fn checker_sort(
    buf: &image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
    crit: Criteria,
) -> image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>> {
    let image_width = buf.width();
    let image_height = buf.height();

    let mut result: image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>> =
        image::ImageBuffer::new(image_width, image_height);

    // TODO make sure an uneven count of checker cells does not crash
    let rows = 100;
    let cols = 100;
    let width = image_width / rows;
    let height = image_height / cols;
    for y in 0..rows {
        for x in 0..cols {
            let x_offset = x * width;
            let y_offset = y * height;
            let sorted_pixels: Vec<(u32, u32, image::Rgb<u8>)> = buf
                .view(x_offset, y_offset, width, height)
                .pixels()
                .sorted_by_key(|p| crit(&p.2))
                .clone()
                .collect();

            let mut sub_image = result.sub_image(x_offset, y_offset, width, height);
            let mut x_ = 0;
            let mut y_ = 0;
            for (_, _, p) in sorted_pixels {
                sub_image.put_pixel(x_, y_, p);
                x_ += 1;
                if x_ >= width {
                    x_ = 0;
                    y_ += 1;
                }
            }
        }
    }

    result
}

fn sort_image(
    sender: &std::sync::mpsc::Sender<u32>,
    sorters: &std::collections::HashMap<String, Sorter>,
    criterias: &std::collections::HashMap<String, Criteria>,
    image_path: &std::path::Path,
) {
    let img_result = image::open(image_path);
    let img = match img_result {
        Err(e) => {
            println!("Could not open image {:?}: {}", image_path, e);
            return;
        }
        Ok(val) => val,
    };

    let img_buf = img.to_rgb();

    let dir_name = match image_path.file_stem() {
        None => return,
        Some(x) => std::path::Path::new(x),
    };
    let dir_path = match image_path.parent() {
        None => {
            println!();
            return;
        }
        Some(x) => x.join(dir_name),
    };
    if !dir_path.is_dir() {
        match std::fs::create_dir(&dir_path) {
            Err(e) => {
                println!("Could not create directory {:?}: {}", dir_name, e);
                return;
            }
            Ok(_) => (),
        }
    }

    for (sort_name, sort) in sorters.iter() {
        for (crit_name, crit) in criterias.iter() {
            let file_path = dir_path.join(format!("{}-{}.png", sort_name, crit_name));

            if file_path.exists() {
                println!("Skipped {:?}", file_path);
                continue;
            }

            let new_img = sort(&img_buf, *crit);
            new_img.save(&file_path).unwrap();
            let size = new_img.width() * new_img.height() * 3;
            match sender.send(size) {
                Ok(_) => (),
                Err(e) => println!("Could not send back written size: {}", e),
            }
            println!("Saved {} bytes - {:?}", size, file_path);
        }
    }
}

fn main() {
    let mut sorters: std::collections::HashMap<String, Sorter> = std::collections::HashMap::new();
    sorters.insert("Basic".to_string(), basic_sort);
    sorters.insert("Checker".to_string(), checker_sort);

    let mut criterias: std::collections::HashMap<String, Criteria> =
        std::collections::HashMap::new();
    criterias.insert("Red".to_string(), get_red);
    criterias.insert("Green".to_string(), get_green);
    criterias.insert("Blue".to_string(), get_blue);
    criterias.insert("Average".to_string(), get_average);
    criterias.insert("Hue".to_string(), get_hue);
    criterias.insert("Saturation".to_string(), get_saturation);
    criterias.insert("Lightness".to_string(), get_lightness);

    let root_dir = std::path::Path::new("res");
    let dirs = match std::fs::read_dir(root_dir) {
        Err(e) => {
            println!("Could not find resource directory: {}", e);
            return;
        }
        Ok(x) => x,
    };

    let (sender, receiver): (std::sync::mpsc::Sender<u32>, std::sync::mpsc::Receiver<u32>) =
        std::sync::mpsc::channel();

    let mut handles = Vec::new();
    for entry in dirs {
        let dir = match entry {
            Err(e) => {
                println!("Could not look at dir entry: {}", e);
                continue;
            }
            Ok(x) => x,
        };
        let image_name = dir.path();
        if image_name.is_dir() {
            continue;
        }
        let s = sorters.clone();
        let c = criterias.clone();
        let send = sender.clone();
        let handle = std::thread::spawn(move || {
            sort_image(&send, &s, &c, &image_name);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut bytes_written = 0;
    loop {
        match receiver.try_recv() {
            Err(_) => break,
            Ok(v) => bytes_written += v,
        }
    }
    let bytes_written_mb = bytes_written as f32 / 1024.0 / 1024.0;
    println!("Wrote {} MB of data", bytes_written_mb);
}
