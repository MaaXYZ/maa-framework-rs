use std::{path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=./cmake/CMakeLists.txt");

    let mut include_dir = vec![];
    let mut lib_dir = vec![];

    if cmake_probe(&mut include_dir, &mut lib_dir).is_err() {
        panic!("Unable to find MaaFramework libraries");
    }

    println!("include_dir: {:?}", include_dir);
    println!("lib_dir: {:?}", lib_dir);

    for dir in &include_dir {
        println!("cargo:include={}", dir.display());
    }

    for dir in &lib_dir {
        println!("cargo:rustc-link-search={}", dir.display());
    }

    println!("cargo:rustc-link-lib=MaaFramework");

    #[cfg(feature = "toolkit")]
    println!("cargo:rustc-link-lib=MaaToolkit");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let bindings_builder = bindgen::Builder::default()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(include_dir.iter().map(|d| format!("-I{}", d.display())));

    let bindings_builder = bindings_builder.header("headers/maa_framework.h");

    #[cfg(feature = "toolkit")]
    let bindings_builder = bindings_builder.header("headers/maa_toolkit.h");

    let bindings = bindings_builder
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn cmake_probe(include_dir: &mut Vec<PathBuf>, libs: &mut Vec<PathBuf>) -> Result<(), ()> {

    let out_dir = std::env::var("OUT_DIR").map_err(|_| ())?;

    let cmake_dir = PathBuf::from(out_dir).join("cmake");

    let cmd = Command::new("cmake")
        .arg("./cmake")
        .arg("-B")
        .arg(cmake_dir)
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .output();

    let output = cmd.map_err(|_| ())?;

    let stderr = String::from_utf8(output.stderr).map_err(|_| ())?;

    println!("{}", stderr);
    for line in stderr.lines() {
        if line.starts_with("IncludeDir") {
            let path = line.split('=').nth(1).ok_or(())?;
            include_dir.push(PathBuf::from(path));
        } else if line.starts_with("MaaFrameworkLibraries") {
            let path = line.split('=').nth(1).ok_or(())?;
            libs.push(PathBuf::from(path).parent().unwrap().to_path_buf());
        }
    }

    Ok(())
}
