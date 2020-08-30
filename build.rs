use std::process::Command;

fn compile_go_module(archive_name: &str, output_path: &str) {
    let output = Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("{}/lib{}.a", output_path, archive_name))
        .arg("gcloud/main.go")
        .output().expect("Failed to run go compiler");
        if !output.status.success() {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            println!("{}", stdout);
            eprintln!("{}", stderr);
        }
        assert!(output.status.success());
}

fn main() {
    let path = "target/gcloud";
    let lib = "gcloud";

    compile_go_module(lib, path);

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}