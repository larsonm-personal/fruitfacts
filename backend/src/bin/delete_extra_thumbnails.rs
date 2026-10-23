// look in /frontend/public/data/ for any thumbnails that don't match a reference .json5 in /plant_database/references/
// and delete them
#[cfg(feature = "binaries")]
use std::fs;

#[cfg(feature = "binaries")]
fn main() {
    let database_dir = harvest_chart_server::import_db::get_database_dir().unwrap();

    let public_dir = database_dir.join("../frontend/public/data/");

    let mut found: i32 = 0;

    for entry in walkdir::WalkDir::new(public_dir.clone())
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();

        if fs::metadata(entry_path).unwrap().is_file() // filenames can't be >260 chars here without help - probably fixed in rust 1.58 - https://github.com/rust-lang/rust/issues/67403
        && (entry_path.to_str().unwrap().ends_with(".jpg") || 
        entry_path.to_str().unwrap().ends_with(".jpg.dvc"))
        {
            let public_absolute_path = fs::canonicalize(&public_dir).unwrap();
            let entry_absolute_path = fs::canonicalize(entry_path).unwrap();
            let entry_relative_path =
                pathdiff::diff_paths(entry_absolute_path, public_absolute_path).unwrap();

            let reference_path = database_dir.join("references").join(entry_relative_path);
            let mut reference_path_string: String = reference_path.to_str().unwrap().to_owned();

            if reference_path_string.ends_with(".jpg") {
                reference_path_string.truncate(reference_path_string.len() - 4);
            } else if reference_path_string.ends_with(".jpg.dvc") {
                reference_path_string.truncate(reference_path_string.len() - 8);
            } else {
                panic!("path with non matching extension");
            }
            let reference_path = std::path::PathBuf::from(reference_path_string + ".json5");

            if !reference_path.exists() {
                println!(
                    "thumbnail without corresponding reference: {} {}",
                    entry_path.display(),
                    reference_path.display()
                );
                found += 1;

                // todo - actually do the delete. maybe a confirmation prompt?
            }
        }
    }

    println!("found {found} extra /frontend/public/data/ files")
}

#[cfg(not(feature = "binaries"))]
fn main() {
    // empty for when the feature isn't selected
    println!("feature \"binaries\" not selected, this tool was not built");
}
