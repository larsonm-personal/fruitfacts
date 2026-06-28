// find all *.pdf recursively in /plant_database/
// for any without an existing thumbnail in /frontend/public/data/, create one
use image::codecs::jpeg::JpegEncoder;
use std::fs;
use std::path::Path;
extern crate clap;
use clap::{crate_version, Arg, Command as ClapApp};

#[cfg(feature = "binaries")]
use pdfium_render::prelude::*;

#[cfg(feature = "binaries")]
fn pdf_first_page_to_jpeg(input_path: &Path, output_path: &Path) -> Result<(), PdfiumError> {
    println!(
        "generating pdf thumbnail for {} at {}",
        input_path.display(),
        output_path.display()
    );
    // adapted from an example in the pdfium_render docs: https://github.com/ajrcarey/pdfium-render

    // Bind to a Pdfium library in the same directory as our Rust executable;
    // failing that, fall back to using a Pdfium library provided by the operating system
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );

    let document = pdfium.load_pdf_from_file(input_path, None)?;

    let render_config = PdfRenderConfig::new()
        .set_target_width(800)
        .set_maximum_height(800);

    for (index, page) in document.pages().iter().enumerate() {
        if index != 0 {
            break;
        }

        let rendered_page = page.render_with_config(&render_config)?;
        let image_data = rendered_page.as_image()?;
        let image = image_data
            .as_rgba8() // ... then converts it to an image::Image...
            .ok_or(PdfiumError::ImageError)?;

        let mut output = fs::File::create(output_path).map_err(|_| PdfiumError::ImageError)?;
        let mut encoder = JpegEncoder::new_with_quality(&mut output, 75); // use an encoder in order to be able to set jpeg quality level
        encoder
            .encode_image(image)
            .map_err(|_| PdfiumError::ImageError)?;

        output.sync_all().map_err(|_| PdfiumError::ImageError)?;
    }

    Ok(())
}

#[cfg(feature = "binaries")]
fn main() {
    println!("generating thumbnails for pdf references");

    let mut found: i32 = 0;
    let mut skipped: i32 = 0;
    let mut errored: i32 = 0;
    let mut done: i32 = 0;

    let matches = ClapApp::new("")
        .version(crate_version!())
        .arg(
            Arg::new("redo_all")
                .short('r')
                .long("redo_all")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("redo all thumbnails"),
        )
        .get_matches();

    let database_dir = harvest_chart_server::import_db::get_database_dir().unwrap();

    for entry in walkdir::WalkDir::new(database_dir.join("references"))
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let input_path = entry.path();

        if fs::metadata(input_path).unwrap().is_file() // filenames can't be >260 chars here without help - probably fixed in rust 1.58 - https://github.com/rust-lang/rust/issues/67403
        && input_path.extension().unwrap().to_str().unwrap() == "pdf"
        {
            found += 1;
            let references_absolute_path =
                fs::canonicalize(database_dir.join("references")).unwrap();
            let reference_absoluate_path = fs::canonicalize(input_path).unwrap();
            let reference_relative_path =
                pathdiff::diff_paths(reference_absoluate_path, references_absolute_path).unwrap();

            let mut output_path = database_dir
                .join("../frontend/public/data/")
                .join(reference_relative_path);
            output_path.set_extension("jpg");

            let mut dir_to_create = output_path.clone();
            let _ = dir_to_create.pop();
            let _ = fs::create_dir_all(dir_to_create);

            if output_path.exists() && !matches.get_flag("redo_all") {
                // println!("jpg already exists");
                skipped += 1;
            } else if let Err(error) = pdf_first_page_to_jpeg(input_path, &output_path) {
                println!("error: {error:?}");
                errored += 1;
            } else {
                done += 1
            }
        }
    }
    println!("found {found}, processed {done} (skipped {skipped} errored {errored})")
}

#[cfg(not(feature = "binaries"))]
fn main() {
    // empty for when the feature isn't selected
    println!("feature \"binaries\" not selected, this tool was not built");
}
