fn main() {
    println!("cargo::rustc-check-cfg=cfg(tracing_pretty)");
    println!("cargo::rerun-if-env-changed=RUST_LOG_PRETTY");
    if std::env::var("RUST_LOG_PRETTY").as_deref() == Ok("1") {
        println!("cargo::rustc-cfg=tracing_pretty");
    }
}
