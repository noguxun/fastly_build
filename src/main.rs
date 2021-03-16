use std::process::Command;
use std::env;
use std::path::Path;


fn main() {
    let root = Path::new("./build-test/");
    env::set_current_dir(&root).unwrap();
    let output = Command::new("cargo").arg("build").output().unwrap();
    println!("{} \n {} \n {} \n", output.status.success(), String::from_utf8(output.stdout).unwrap(), String::from_utf8(output.stderr).unwrap());
}
