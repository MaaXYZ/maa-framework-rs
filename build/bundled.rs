use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

const MAA_FRAMEWORK_VERSION: &str = "v1.7.0-alpha.3";

pub fn get_bundled_dir(include_dir: &mut Vec<PathBuf>, libs: &mut Vec<PathBuf>) -> Result<(), ()> {
    let (include_path, lib_path) = download_maa_framework().unwrap();

    include_dir.push(include_path);
    libs.push(lib_path);

    Ok(())
}

fn download_maa_framework() -> Result<(PathBuf, PathBuf), ()> {
    let system = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        return Err(());
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        return Err(());
    };

    let download_url = format!("https://github.com/MaaAssistantArknights/MaaFramework/releases/download/{MAA_FRAMEWORK_VERSION}/MAA-{system}-{arch}-{MAA_FRAMEWORK_VERSION}.zip");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let download_dir = PathBuf::from(out_dir).join("maa_framework");
    if !download_dir.exists() {
        std::fs::create_dir(&download_dir).unwrap();
    }

    let mut content = ureq::get(&download_url).call().unwrap().into_reader();

    let output_zip_path = download_dir.join("maa_framework.zip");
    let mut output_zip = std::fs::File::create(&output_zip_path).unwrap();

    std::io::copy(&mut content, &mut output_zip).unwrap();

    let zip = File::open(&output_zip_path).unwrap();
    let mut zip = zip::ZipArchive::new(zip).unwrap();

    let include_dir = download_dir.join("include");
    create_dir_all(download_dir.join(&include_dir)).unwrap();

    let lib_dir = download_dir.join("lib");
    create_dir_all(download_dir.join(&lib_dir)).unwrap();

    // extract include dir
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();

        if file.is_dir() {
            continue;
        }

        let file_path = file.mangled_name();

        let mut extract_files_in = |dir: &str, dst: &PathBuf| {
            if !file_path.starts_with(dir) {
                return;
            }
            let file_path = file_path.strip_prefix(dir).unwrap();
            let file_path = dst.join(file_path);
            create_dir_all(file_path.parent().unwrap()).unwrap();
            let mut output_file = std::fs::File::create(&file_path).unwrap();
            std::io::copy(&mut file, &mut output_file).unwrap();
        };

        extract_files_in("include", &include_dir);

        #[cfg(target_os = "windows")]
        extract_files_in("lib", &lib_dir);

        #[cfg(not(target_os = "windows"))]
        extract_files_in("bin", &lib_dir);
    }

    Ok((include_dir, lib_dir))
}
