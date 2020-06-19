extern crate image;
extern crate itertools;
extern crate num_complex;
use crate::image::GenericImage;
use crate::image::GenericImageView;
use itertools::Itertools;

type Criteria = fn(&image::Rgb<u8>) -> u8;

fn get_red(item: &image::Rgb<u8>) -> u8 {
    item.0[0]
}
fn get_green(item: &image::Rgb<u8>) -> u8 {
    item.0[1]
}
fn get_blue(item: &image::Rgb<u8>) -> u8 {
    item.0[2]
}
fn get_average(item: &image::Rgb<u8>) -> u8 {
    (item.0[0] + item.0[1] + item.0[2]) / 3
}

type Sorter = fn(
    buf: &image::ImageBuffer<image::Rgb<u8>, std::vec::Vec<u8>>,
    crit: fn(&image::Rgb<u8>) -> u8,
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

    let width = image_width / 10;
    let height = image_height / 10;
    for y in 0..10 {
        for x in 0..10 {
            let x_offset = x * width;
            let y_offset = y * height;
            let sorted_pixels: Vec<(u32, u32, image::Rgb<u8>)> = buf
                .view(x_offset, y_offset, width, height)
                .pixels()
                .sorted_by_key(|p| crit(&p.2))
                /*.flat_map(|p| {
                    let pixel = p.2;
                    let mut data = Vec::new();
                    for a in pixel.0.iter() {
                        data.push(*a);
                    }
                    data
                })*/
                .clone()
                .collect();
            let mut sub_image = result.sub_image(x_offset, y_offset, width, height);
            for (x_, y_, p) in sorted_pixels {
                sub_image.put_pixel(x_, y_, p);
            }
        }
    }

    result
}

fn sort_image(
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
            println!("Saved {:?}", file_path);
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

    let root_dir = std::path::Path::new("res");
    let dirs = match std::fs::read_dir(root_dir) {
        Err(e) => {
            println!("Could not find resource directory: {}", e);
            return;
        }
        Ok(x) => x,
    };

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
        let handle = std::thread::spawn(move || {
            sort_image(&s, &c, &image_name);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
