use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let source = "src/constants/params.json";
    let profile = env::var("PROFILE").unwrap();
    
    let mut dest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    dest.push("target");
    dest.push(&profile);
    dest.push("params.json");

    fs::copy(source, &dest).expect("Failed to copy params.json");
    
    println!("cargo:rerun-if-changed={}", source);
    println!("Copied {} to {:?}", source, dest);
}
