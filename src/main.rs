extern crate image;
extern crate itertools;
extern crate num_complex;
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
        },
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

    let mut criterias: std::collections::HashMap<String, Criteria> =
        std::collections::HashMap::new();
    criterias.insert("Red".to_string(), get_red);
    criterias.insert("Green".to_string(), get_green);
    criterias.insert("Blue".to_string(), get_blue);

    let root_dir = std::path::Path::new("res");
    let dirs = match std::fs::read_dir(root_dir) {
        Err(e) => {
            println!("Could not find resource directory: {}", e);
            return;
        }
        Ok(x) => x,
    };
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
        sort_image(&sorters, &criterias, &image_name)
    }
}
