use anyhow::{anyhow, Result};
use rand::{distributions::Alphanumeric, rngs::SmallRng, Rng, SeedableRng};
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::process::Command;

fn main() {
    let mut crate_dep = r#"rust-crypto = "^0.3""#;
    let mut result = build_crate_wasm32(crate_dep).unwrap();
    println!("{} --> {}", crate_dep, result);

    let mut crate_dep =     r#"rust-crypto-wasm = "^0.2""#;
    let mut result = build_crate_wasm32(crate_dep).unwrap();
    println!("{} --> {}", crate_dep, result);
}

fn build_crate_wasm32(crate_dep: &str) -> Result<bool> {
    // create new project folder name
    let rand_string: String = SmallRng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let new_build_folder = format!("build-test-{}", rand_string);

    // copy template project to a new folder
    let mut output = Command::new("cp")
        .arg("-a")
        .arg("build-test")
        .arg(&new_build_folder)
        .output()?;

    if !output.status.success() {
        let err_msg = format!(
            "{} {}",
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        );
        return Err(anyhow!(err_msg));
    }

    // cd into new folder
    env::set_current_dir(&new_build_folder)?;

    // append the crate dependency to cargo.toml
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("Cargo.toml")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", crate_dep) {
        eprintln!("Couldn't write to file: {}", e);
    }

    // kick off the build
    output = Command::new("cargo").arg("build").output()?;

    /*
    println!(
        "{} \n {} \n {} \n",
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    );
    */

    // get out of new directory
    env::set_current_dir("..")?;

    // remove the new directory
    Command::new("rm")
        .arg("-rf")
        .arg(&new_build_folder)
        .output()?;

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_crate_wasm32_works() {
        assert_eq!(
            build_crate_wasm32(r#"rust-crypto = "^0.3""#).unwrap(),
            false
        );
        assert_eq!(
            build_crate_wasm32(r#"rust-crypto-wasm = "^0.2""#).unwrap(),
            true
        );
    }
}
