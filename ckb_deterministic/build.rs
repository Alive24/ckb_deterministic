use std::process::Command;
use std::fs;

fn main() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cd ../ && moleculec --language rust --schema-file schemas/deterministic.mol")
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        panic!("moleculec failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    fs::create_dir_all("src/generated").expect("Unable to create generated directory");
    fs::write("src/generated/deterministic.rs", &output.stdout).expect("Unable to write file");
}