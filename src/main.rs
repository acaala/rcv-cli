use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process,
};

use anyhow::{anyhow, bail, Context, Error, Result};
use clap::{arg, command, Parser};
use image::ImageError;
use webp::Encoder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = None)]
    input_file: Option<String>,

    #[arg(short, long, default_value = None)]
    directory: Option<String>,

    #[arg(short, long)]
    output_path: String,

    #[arg(short, long, default_value_t = 75f32)]
    quality: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.input_file.is_none() && args.directory.is_none() {
        eprintln!("Error: An input file (--input) or a directory (--directory) must be used");
        process::exit(0)
    }

    let output_path = Path::new(&args.output_path);
    fs::create_dir_all(&output_path).unwrap_or_else(|_| {
        eprintln!("Failed to create output path");
        process::exit(1)
    });

    match args.directory.is_some() {
        true => {
            if let Err(_) = process_directory(&args.directory.unwrap(), output_path, args.quality) {
                bail!("Error: Failed to open directory");
            }
        }
        false => {
            if let Err(err) = process_image(&args.input_file.unwrap(), &output_path, args.quality) {
                bail!("Failed to process file - {:?}", err);
            }
        }
    }

    Ok(())
}

fn process_image(input_file: &str, output_path: &Path, quality: f32) -> Result<(), Error> {
    let image_path = Path::new(input_file);
    let file_size = fs::metadata(image_path).unwrap().len();

    let img = image::open(image_path)?;

    let file_name = image_path.file_name().unwrap_or_else(|| {
        println!("Cannot get name from file using default");
        OsStr::new("default")
    });

    println!("Converting {:?}", file_name);

    let encoder = Encoder::from_image(&img).map_err(|_| anyhow!("Failed to create encoder"))?;

    let webp = encoder.encode(quality);

    let output_path = output_path.join(file_name).with_extension("webp");
    fs::write(&output_path, &*webp).unwrap();

    let new_file_size = fs::metadata(output_path).unwrap().len();
    let percentage_change =
        ((file_size as f64 - new_file_size as f64) / file_size as f64) * 100 as f64;
    println!(
        "Saved {:?} KB ({:?}%)",
        (file_size - new_file_size) / 1024,
        percentage_change as u64
    );

    Ok(())
}

fn get_files_in_dir(dir: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_image_file(&path) {
            files.push(path);
        }
    }

    Ok(files)
}

fn is_image_file(file: &Path) -> bool {
    image::open(file).is_ok()
}

fn process_directory(dir: &str, output_path: &Path, quality: f32) -> Result<()> {
    let files = get_files_in_dir(&dir)?;

    for file in files {
        if let Err(_) = process_image(file.to_str().unwrap(), output_path, quality) {
            eprintln!(
                "Error processing file: {:?} - Skipping...",
                file.file_name().unwrap()
            );
        }
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_image() {
        let input_file = "./test_assets/test_img.jpg";
        let output_path = Path::new("./assets");
        let quality = 70.0;

        let result = process_image(input_file, output_path, quality);

        assert!(result.is_ok())
    }

    #[test]
    fn test_process_directory() {
        let directory = "test_assets";
        let output_path = Path::new("./assets");
        let quality = 70.0;

        let result = process_directory(directory, output_path, quality);

        assert!(result.is_ok())
    }
}
