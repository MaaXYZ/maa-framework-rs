#[path = "build/bundled.rs"]
mod bundled;
#[path = "build/cmake_probe.rs"]
mod cmake_probe;

fn main() {

    if std::env::var("DOCS_RS").is_ok() {
        // skip building on docs.rs
        return;
    }

    println!("cargo:rerun-if-changed=./cmake/CMakeLists.txt");

    let mut include_dir = vec![];
    let mut lib_dir = vec![];

    #[cfg(not(feature = "bundled"))]
    if cmake_probe::cmake_probe(&mut include_dir, &mut lib_dir).is_err() {
        panic!("Unable to find MaaFramework libraries");
    }

    #[cfg(feature = "download")]
    if bundled::get_bundled_dir(&mut include_dir, &mut lib_dir).is_err() {
        panic!("Unable to download MaaFramework libraries");
    }

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
