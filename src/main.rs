use futures::executor::block_on;
use glob::glob;
use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout}; // Using image crate: https://github.com/image-rs/image
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use usvg::Tree;
use webp::{Encoder, WebPMemory}; // Using webp crate: https://github.com/jaredforth/webp

pub async fn image_to_webp(file_path: &String) {
    // Open path as DynamicImage
    let image = ImageReader::open(file_path);
    let image: DynamicImage = match image {
        Ok(img) => img.with_guessed_format().unwrap().decode().unwrap(), //ImageReader::with_guessed_format() function guesses if image needs to be opened in JPEG or PNG format.
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    // Make webp::Encoder from DynamicImage.
    let encoder: Encoder = Encoder::from_image(&image).unwrap();

    // Encode image into WebPMemory.
    let encoded_webp: WebPMemory = encoder.encode(65f32);

    // Put webp-image in output folder with the same file structure
    let path: &Path = Path::new(file_path);
    let parent_directory: &Path = path.parent().unwrap();
    let webp_folder_path = format!("output/{}", parent_directory.to_str().unwrap());
    match std::fs::create_dir_all(webp_folder_path.to_string()) {
        Ok(_) => {}
        Err(e) => {
            println!("Error {}", e);
        }
    }

    // Get filename of original image.
    let filename_original_image = path.file_stem().unwrap().to_str().unwrap();

    // Make full output path for webp-image.
    let webp_image_path = format!(
        "{}/{}.webp",
        webp_folder_path.to_string(),
        filename_original_image
    );

    let mut webp_image = File::create(webp_image_path.to_string()).unwrap();
    match webp_image.write_all(encoded_webp.as_bytes()) {
        Err(e) => {
            println!("Error: {}", e);
        }
        Ok(_) => {
            println!("Converted: {:?}", webp_image_path)
        }
    }
}
pub async fn svg_to_webp(image_path: &String) {
    let path: &Path = Path::new(&image_path);
    let parent_directory: &Path = path.parent().unwrap();
    let webp_folder_path = format!("output/{}", parent_directory.to_str().unwrap());

    //read file
    let contents = read_to_string(path).expect("Error reading file");
    //parse file
    let opt = usvg::Options::default();
    let tree = Tree::from_str(&contents, &opt.to_ref()).unwrap();
    let pixmap_size = tree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    // Get filename of original image.
    let filename_original_image = path.file_stem().unwrap().to_str().unwrap();

    // Make full output path for webp-image.
    let webp_image_path = format!(
        "{}/{}.webp",
        webp_folder_path.to_string(),
        filename_original_image
    );

    pixmap.save_png(&webp_image_path).unwrap();
    println!("Converted: {:?}", &webp_image_path);
}

fn main() {
    println!("Starting...");
    for entry in glob("images/**/[!.]*.*").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let path_string = path.to_str().unwrap().to_string();
                match Path::new(&path_string)
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                {
                    "svg" => {
                        block_on(svg_to_webp(&path_string));
                    }
                    "png" | "jpg" | "jpeg" => {
                        block_on(image_to_webp(&path_string));
                    }
                    _ => {
                        println!("Error wrong format: {:?}", path_string);
                    }
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
