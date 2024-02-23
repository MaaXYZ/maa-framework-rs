use std::{path::PathBuf, process::Command};

pub fn cmake_probe(include_dir: &mut Vec<PathBuf>, libs: &mut Vec<PathBuf>) -> Result<(), ()> {
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
