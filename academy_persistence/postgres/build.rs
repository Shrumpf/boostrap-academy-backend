use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo::rerun-if-changed=migrations");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let migrations_rs = out_dir.join("migrations.rs");
    emit_migrations(&migrations_rs);
    println!("cargo::rustc-env=MIGRATIONS={}", migrations_rs.display());
}

fn emit_migrations(path: &Path) {
    let file = std::fs::File::create(path).unwrap();
    let mut writer = BufWriter::new(file);
    write!(&mut writer, "&[").unwrap();
    for (name, (up, down)) in collect_migrations() {
        write!(
            &mut writer,
            "Migration{{name:{name:?},up:{up:?},down:{down:?}}},"
        )
        .unwrap();
    }
    write!(&mut writer, "]").unwrap();
    writer.flush().unwrap();
}

fn collect_migrations() -> BTreeMap<String, (String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");

    let mut out = BTreeMap::new();
    for file in dir.read_dir().unwrap() {
        let file = file.unwrap();
        let name = file.file_name().into_string().unwrap();
        if let Some(name) = name.strip_suffix(".up.sql").map(ToOwned::to_owned) {
            let content = std::fs::read_to_string(file.path()).unwrap();
            if let Some((up, _)) = out.get_mut(&name) {
                *up = content;
            } else {
                out.insert(name, (content, String::new()));
            }
        } else if let Some(name) = name.strip_suffix(".down.sql").map(ToOwned::to_owned) {
            let content = std::fs::read_to_string(file.path()).unwrap();
            if let Some((_, down)) = out.get_mut(&name) {
                *down = content;
            } else {
                out.insert(name, (String::new(), content));
            }
        }
    }
    out
}
