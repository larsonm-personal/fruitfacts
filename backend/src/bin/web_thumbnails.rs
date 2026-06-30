// walk through all .json5 references to find any marked "thumbnail: website"
// for any without an existing thumbnail,
// call out to /backend/web_screenshot/index.js to generate puppeteer screenshots for their reference urls

use std::fs;
use std::path::Path;
extern crate clap;
use anyhow::{anyhow, Result};
use clap::{crate_version, Arg, Command as ClapApp};
use std::io::Write;

#[cfg(windows)]
fn node_compatible_path(path: &Path) -> String {
    let path_text = path.to_string_lossy();
    if let Some(rest) = path_text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = path_text.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        path_text.into_owned()
    }
}

#[cfg(not(windows))]
fn node_compatible_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(feature = "binaries")]
fn web_address_to_jpg(web_address: &str, script_path: &str, output_path: &Path) -> Result<()> {
    println!(
        "generating thumbnail for {web_address} at {}",
        output_path.display()
    ); // todo remove

    let output = std::process::Command::new("node")
        .arg(script_path)
        .arg(web_address)
        .arg(output_path)
        .output()
        .expect("failed to run node screenshot process");

    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    if output.status.code() != Some(0) {
        return Err(anyhow!("screenshot process failed"));
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct CollectionJson {
    thumbnail: Option<String>,
    url: Option<String>,
}

#[cfg(feature = "binaries")]
fn main() {
    println!("generating thumbnails for website references");

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

    let binding =
        fs::canonicalize(database_dir.join("../backend/web_screenshot/index.js")).unwrap();
    let script_path = node_compatible_path(binding.as_path());

    for entry in walkdir::WalkDir::new(database_dir.join("references"))
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let input_path = entry.path();

        if fs::metadata(input_path).unwrap().is_file() // filenames can't be >260 chars here without help - probably fixed in rust 1.58 - https://github.com/rust-lang/rust/issues/67403
        && input_path.extension().unwrap().to_str().unwrap() == "json5"
        {
            let contents = fs::read_to_string(input_path).unwrap();

            let collection: CollectionJson = json5::from_str(&contents).unwrap_or_else(|error| {
                panic!(
                    "couldn't parse json in file {} {}",
                    input_path.display(),
                    error
                );
            });

            if collection.thumbnail != Some("website".to_string()) {
                continue; // if not marked, skip, maybe it has a pdf screenshot instead
            }

            // todo - fetch web address from the "url" field (and use only the first part before spaces)

            let web_address: String = collection
                .url
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .to_string();

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
            } else if let Err(error) = web_address_to_jpg(&web_address, &script_path, &output_path)
            {
                println!("error: {error:?}");
                errored += 1;
            } else {
                done += 1;
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
