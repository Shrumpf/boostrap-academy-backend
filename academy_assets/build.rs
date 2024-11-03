use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo::rerun-if-changed=assets");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let out_path = out_dir.join("assets.rs");
    let mut out = std::fs::File::create(&out_path).unwrap();
    emit_assets(&mut out, &assets);
    println!("cargo::rustc-env=ASSETS={}", out_path.display());
}

fn emit_assets(out: &mut File, assets: &Path) {
    for asset in assets.read_dir().unwrap() {
        let asset = asset.unwrap();
        let name = asset.file_name().into_string().unwrap();
        let file_type = asset.file_type().unwrap();
        let path = asset.path();

        if name.starts_with(".") {
            continue;
        }

        if file_type.is_dir() {
            writeln!(out, "pub mod {} {{", to_snake_case(&name)).unwrap();
            emit_assets(out, &path);
            writeln!(out, "}}").unwrap();
        } else if file_type.is_file() {
            if std::fs::read_to_string(&path).is_ok() {
                emit_str_asset(out, &name, &path);
            } else {
                emit_bytes_asset(out, &name, &path);
            }
        }
    }
}

fn emit_bytes_asset(out: &mut File, name: &str, path: &Path) {
    writeln!(
        out,
        "pub const {}: &[u8] = ::core::include_bytes!({:?});",
        to_screaming_snake_case(name),
        path,
    )
    .unwrap();
}

fn emit_str_asset(out: &mut File, name: &str, path: &Path) {
    writeln!(
        out,
        "pub const {}: &str = ::core::include_str!({:?});",
        to_screaming_snake_case(name),
        path,
    )
    .unwrap();
}

fn to_screaming_snake_case(name: &str) -> String {
    to_snake_case(name).to_uppercase()
}

fn to_snake_case(name: &str) -> String {
    let mut words = name
        .split(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
        .map(|word| word.to_lowercase());
    let mut out = words.next().unwrap_or_default();
    for w in words {
        out.push('_');
        out.push_str(&w);
    }
    out
}
