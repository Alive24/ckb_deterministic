use std::fs;
use std::process::Command;

fn main() {
    // Declare custom cfg attributes to avoid "unexpected cfg" warnings
    println!("cargo:rustc-check-cfg=cfg(feature, values(\"native-simulator\"))");
    println!("cargo:rustc-check-cfg=cfg(feature, values(\"library\"))");

    let output = Command::new("sh")
        .arg("-c")
        .arg("cd ../ && moleculec --language rust --schema-file schemas/deterministic.mol")
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        panic!(
            "moleculec failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fs::create_dir_all("src/generated").expect("Unable to create generated directory");
    fs::write("src/generated/deterministic.rs", &output.stdout).expect("Unable to write file");
}
