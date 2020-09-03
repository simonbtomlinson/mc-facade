use std::{env, path::PathBuf, process::Command};

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn out_file(name: impl Into<String>) -> PathBuf {
    out_dir().join(name.into())
}

fn compile_go_module(archive_name: &str) {
    let output = Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(out_file(format!("lib{}.a", archive_name)))
        .arg("gcloud/main.go")
        .output()
        .expect("Failed to run go compiler");
    if !output.status.success() {
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        println!("{}", stdout);
        eprintln!("{}", stderr);
    }
    assert!(output.status.success());
}

fn run_bindgen(go_header: &str) {
    let bindings = bindgen::Builder::default()
        .header(out_file(format!("lib{}.h", go_header)).to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(out_file("bindings.rs"))
        .expect("Failed to write bindings");
}

fn main() {
    let path = out_dir();
    let lib = "gcloud";

    compile_go_module(lib);

    run_bindgen(lib);

    println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    println!("cargo:rustc-link-lib=static={}", lib);
}
