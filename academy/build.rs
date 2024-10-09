fn main() {
    println!("cargo::rustc-check-cfg=cfg(tracing_pretty)");
    println!("cargo::rerun-if-env-changed=ACADEMY_DEVENV");
    println!("cargo::rerun-if-env-changed=RUST_LOG_PRETTY");

    let devenv = load_bool_from_env("ACADEMY_DEVENV", false);
    let pretty = load_bool_from_env("RUST_LOG_PRETTY", devenv);
    if pretty {
        println!("cargo::rustc-cfg=tracing_pretty");
    }
}

fn load_bool_from_env(var: &str, default: bool) -> bool {
    match std::env::var(var).as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        _ => default,
    }
}
