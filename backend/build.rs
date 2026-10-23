//extern crate dotenv_build;
use std::process::Command;

fn main() {
    // set up env vars for this build based on env files, which are then picked up by the std::env!() compile time macro
    // this saves having to load env files with some kind of cross-platform script

    // detecting debug/release: see https://stackoverflow.com/questions/57296104/how-to-access-current-cargo-profile-debug-release-from-the-build-script-bu
    let profile = std::env::var("PROFILE").unwrap();
    match profile.as_str() {
        profile if profile == "release" || profile == "debug" => {
            dotenv_build::output_multiple(vec![
                dotenv_build::Config {
                    filename: std::path::Path::new(&format!(".env.{profile}")),
                    fail_if_missing_dotenv: true,
                    ..Default::default()
                },
                // load .env
                dotenv_build::Config {
                    fail_if_missing_dotenv: true,
                    ..Default::default()
                },
            ])
            .unwrap();
        }
        profile => {
            panic!("unknown build profile {profile}")
        }
    }

    // also gather build-time data and put it into some extra env vars
    {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        println!(
            "cargo:rustc-env=GIT_HASH={}",
            String::from_utf8(output.stdout).unwrap()
        );
    }
    {
        let output = Command::new("git")
            .args(["show", "-s", "--format=%ct"])
            .output()
            .unwrap();
        println!(
            "cargo:rustc-env=GIT_UNIX_TIME={}",
            String::from_utf8(output.stdout).unwrap()
        );
    }
    {
        let output = Command::new("git")
            .args(["rev-list", "--count", "main"])
            .output()
            .unwrap();
        println!(
            "cargo:rustc-env=GIT_MAIN_COMMIT_COUNT={}",
            String::from_utf8(output.stdout).unwrap()
        );
    }
}
