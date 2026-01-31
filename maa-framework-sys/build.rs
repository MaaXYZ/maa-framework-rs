use std::{path::PathBuf, process::Command};

/// Try to locate MaaFramework using CMake's find_package.
fn cmake_probe(include_dir: &mut Vec<PathBuf>, libs: &mut Vec<PathBuf>) -> Result<(), ()> {
    let out_dir = std::env::var("OUT_DIR").map_err(|_| ())?;
    let cmake_dir = PathBuf::from(out_dir).join("cmake");

    let cmd = Command::new("cmake")
        .arg("./cmake")
        .arg("-B")
        .arg(&cmake_dir)
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .output();

    let output = cmd.map_err(|_| ())?;
    let stderr = String::from_utf8(output.stderr).map_err(|_| ())?;

    for line in stderr.lines() {
        if line.starts_with("IncludeDir") {
            let path = line.split('=').nth(1).ok_or(())?;
            include_dir.push(PathBuf::from(path));
        } else if line.starts_with("MaaFrameworkLibraries") {
            let path = line.split('=').nth(1).ok_or(())?;
            let p = PathBuf::from(path);
            if let Some(parent) = p.parent() {
                if parent.as_os_str().is_empty() {
                    libs.push(PathBuf::from("."));
                } else {
                    libs.push(parent.to_path_buf());
                }
            }
        }
    }

    Ok(())
}

/// Probe a directory for MaaFramework SDK structure (include/, lib/, bin/).
fn probe_sdk_dir(dir: &PathBuf, include_dir: &mut Vec<PathBuf>, lib_dir: &mut Vec<PathBuf>) {
    let sdk_include = dir.join("include");
    let sdk_lib = dir.join("lib");
    let sdk_bin = dir.join("bin");

    if sdk_include.exists() && !include_dir.iter().any(|d| d == &sdk_include) {
        include_dir.push(sdk_include);
    }
    if sdk_lib.exists() {
        lib_dir.push(sdk_lib);
    }
    if sdk_bin.exists() {
        lib_dir.push(sdk_bin.clone());
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=MAA_SDK_PATH");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DYNAMIC");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_GENERATE_BINDINGS");

    let is_dynamic = std::env::var("CARGO_FEATURE_DYNAMIC").is_ok();
    let is_static = std::env::var("CARGO_FEATURE_STATIC").is_ok();

    if is_dynamic && is_static {
        println!("cargo:warning=MaaFramework: Both 'static' and 'dynamic' features are enabled. Forcing 'dynamic' mode.");
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Check for pre-generated bindings
    let bindings_dir = manifest_dir.join("src/bindings");
    let static_bindings_src = bindings_dir.join("static_bindings.rs");
    let dynamic_bindings_src = bindings_dir.join("dynamic_bindings.rs");
    let shims_src = bindings_dir.join("shims.rs");

    let bindings_dst = out_dir.join("bindings.rs");
    let shims_dst = out_dir.join("shims.rs");

    // Determine if we should use pre-generated bindings
    let is_docs_rs = std::env::var("DOCS_RS").is_ok();
    let regenerate_bindings =
        !is_docs_rs && std::env::var("CARGO_FEATURE_GENERATE_BINDINGS").is_ok();

    let use_pregenerated = !regenerate_bindings && {
        if is_dynamic {
            dynamic_bindings_src.exists() && shims_src.exists()
        } else {
            static_bindings_src.exists()
        }
    };

    let mut include_dir = vec![];
    let mut lib_dir = vec![];

    if regenerate_bindings || !is_docs_rs {
        // Priority 1: Environment variable MAA_SDK_PATH
        if let Ok(sdk_path) = std::env::var("MAA_SDK_PATH") {
            let sdk_dir = PathBuf::from(&sdk_path);
            if sdk_dir.exists() {
                println!("cargo:warning=Using SDK from MAA_SDK_PATH: {}", sdk_path);
                probe_sdk_dir(&sdk_dir, &mut include_dir, &mut lib_dir);
            }
        }

        // Determine user's project root from OUT_DIR
        // OUT_DIR is typically: {user_project}/target/{profile}/build/{pkg}/out
        // So we go up to find the project root (where Cargo.toml is)
        let user_project_root = out_dir
            .ancestors()
            .find(|p| p.join("Cargo.toml").exists())
            .map(|p| p.to_path_buf());

        // Priority 2: MAA-* directories in user's project root
        if include_dir.is_empty() {
            if let Some(ref project_root) = user_project_root {
                for entry in std::fs::read_dir(project_root).into_iter().flatten() {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir()
                            && path
                                .file_name()
                                .map_or(false, |n| n.to_string_lossy().starts_with("MAA-"))
                        {
                            println!(
                                "cargo:warning=Using SDK from user project: {}",
                                path.display()
                            );
                            probe_sdk_dir(&path, &mut include_dir, &mut lib_dir);
                        }
                    }
                }
            }
        }

        // Priority 3: sdk/ directory in user's project root
        if include_dir.is_empty() {
            if let Some(ref project_root) = user_project_root {
                let sdk = project_root.join("sdk");
                if sdk.join("include").exists() || sdk.join("bin").exists() {
                    println!(
                        "cargo:warning=Using SDK from user project sdk/: {}",
                        sdk.display()
                    );
                    probe_sdk_dir(&sdk, &mut include_dir, &mut lib_dir);
                }
            }
        }

        // Priority 4: install/ directory in user's project root
        if include_dir.is_empty() {
            if let Some(ref project_root) = user_project_root {
                let install = project_root.join("install");
                if install.exists() {
                    println!(
                        "cargo:warning=Using SDK from user project install/: {}",
                        install.display()
                    );
                    probe_sdk_dir(&install, &mut include_dir, &mut lib_dir);
                }
            }
        }

        // Priority 5: MAA-* directories in crate's manifest directory (for development)
        if include_dir.is_empty() {
            for entry in std::fs::read_dir(&manifest_dir).into_iter().flatten() {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir()
                        && path
                            .file_name()
                            .map_or(false, |n| n.to_string_lossy().starts_with("MAA-"))
                    {
                        probe_sdk_dir(&path, &mut include_dir, &mut lib_dir);
                    }
                }
            }
        }

        // Priority 6: sdk/ directory in crate's manifest directory
        if include_dir.is_empty() {
            let sdk = manifest_dir.join("sdk");
            if sdk.join("include").exists() {
                probe_sdk_dir(&sdk, &mut include_dir, &mut lib_dir);
            }
        }

        // Priority 7: install/ directory (local builds)
        if include_dir.is_empty() {
            let install = manifest_dir.join("install");
            if install.exists() {
                probe_sdk_dir(&install, &mut include_dir, &mut lib_dir);
            }
        }

        // Priority 8: In-tree build (MaaFramework/source/binding/Rust)
        if include_dir.is_empty() {
            for parent in ["../install", "../../install", "../../../install"] {
                let install = manifest_dir.join(parent);
                if install.exists() {
                    probe_sdk_dir(
                        &install.canonicalize().unwrap_or(install),
                        &mut include_dir,
                        &mut lib_dir,
                    );
                    break;
                }
            }
        }

        // Priority 9: CMake find_package
        if include_dir.is_empty() {
            let _ = cmake_probe(&mut include_dir, &mut lib_dir);
        }
    }

    // Handle bindings generation or copying
    if use_pregenerated {
        // Use pre-generated bindings
        println!("cargo:warning=Using pre-generated bindings from src/bindings/");

        if is_dynamic {
            std::fs::copy(&dynamic_bindings_src, &bindings_dst)
                .expect("Failed to copy pre-generated dynamic bindings");
            std::fs::copy(&shims_src, &shims_dst).expect("Failed to copy pre-generated shims");
        } else {
            std::fs::copy(&static_bindings_src, &bindings_dst)
                .expect("Failed to copy pre-generated static bindings");
        }
    } else if regenerate_bindings {
        // Regenerate bindings using bindgen
        #[cfg(feature = "generate-bindings")]
        {
            println!("cargo:warning=Regenerating bindings with bindgen...");

            if include_dir.is_empty() {
                println!("cargo:warning===========================================");
                println!("cargo:warning=MaaFramework SDK not found!");
                println!("cargo:warning=");
                println!("cargo:warning=Please download the SDK from:");
                println!("cargo:warning=  https://github.com/MaaXYZ/MaaFramework/releases");
                println!("cargo:warning=");
                println!("cargo:warning=Then either:");
                println!("cargo:warning=  1. Extract to project root (MAA-* directory)");
                println!("cargo:warning=  2. Set MAA_SDK_PATH environment variable");
                println!("cargo:warning===========================================");
                panic!("MaaFramework SDK not found. See warnings above.");
            }

            generate_bindings_with_bindgen(&include_dir, &out_dir, is_dynamic);
        }
    } else if is_docs_rs {
        // docs.rs build: try to use pre-generated, otherwise fail gracefully
        println!("cargo:warning=Detected docs.rs build, using pre-generated bindings.");

        if is_dynamic && dynamic_bindings_src.exists() && shims_src.exists() {
            std::fs::copy(&dynamic_bindings_src, &bindings_dst)
                .expect("Failed to copy pre-generated dynamic bindings");
            std::fs::copy(&shims_src, &shims_dst).expect("Failed to copy pre-generated shims");
        } else if !is_dynamic && static_bindings_src.exists() {
            std::fs::copy(&static_bindings_src, &bindings_dst)
                .expect("Failed to copy pre-generated static bindings");
        } else {
            println!("cargo:warning=Pre-generated bindings not found!");
            // Create empty bindings to allow docs.rs to at least show something
            std::fs::write(&bindings_dst, "// Bindings not available on docs.rs\n")
                .expect("Failed to write placeholder bindings");
        }
    } else {
        // No pre-generated bindings and not regenerating - use pre-generated (should exist)
        if is_dynamic {
            if dynamic_bindings_src.exists() && shims_src.exists() {
                std::fs::copy(&dynamic_bindings_src, &bindings_dst)
                    .expect("Failed to copy pre-generated dynamic bindings");
                std::fs::copy(&shims_src, &shims_dst).expect("Failed to copy pre-generated shims");
            } else {
                println!("cargo:warning=Pre-generated dynamic bindings not found!");
                println!("cargo:warning=Run with --features generate-bindings to generate them.");
                panic!("Pre-generated dynamic bindings not found.");
            }
        } else {
            if static_bindings_src.exists() {
                std::fs::copy(&static_bindings_src, &bindings_dst)
                    .expect("Failed to copy pre-generated static bindings");
            } else {
                println!("cargo:warning=Pre-generated static bindings not found!");
                println!("cargo:warning=Run with --features generate-bindings to generate them.");
                panic!("Pre-generated static bindings not found.");
            }
        }
    }

    if !is_docs_rs {
        // Output library search paths
        for dir in &lib_dir {
            println!("cargo:rustc-link-search=native={}", dir.display());

            if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
            } else if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default() == "unix" {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
            }
        }

        if !is_dynamic {
            println!("cargo:rustc-link-lib=MaaFramework");
            println!("cargo:rustc-link-lib=MaaAgentClient");
            println!("cargo:rustc-link-lib=MaaAgentServer");

            #[cfg(feature = "toolkit")]
            println!("cargo:rustc-link-lib=MaaToolkit");
        }

        copy_dlls_to_target(&lib_dir, &out_dir);
    }
}

/// Generate bindings using bindgen (requires LLVM/Clang)
#[cfg(feature = "generate-bindings")]
fn generate_bindings_with_bindgen(include_dir: &[PathBuf], out_path: &PathBuf, is_dynamic: bool) {
    let clang_include_args = include_dir.iter().map(|d| {
        let s = d.to_string_lossy();
        if cfg!(windows) && s.starts_with(r"\\?\") {
            format!("-I{}", &s[4..])
        } else {
            format!("-I{}", s)
        }
    });

    let mut bindings_builder = bindgen::Builder::default()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(clang_include_args)
        .header("headers/wrapper.h");

    #[cfg(feature = "toolkit")]
    {
        bindings_builder = bindings_builder.header("headers/maa_toolkit.h");
    }

    bindings_builder = bindings_builder.blocklist_function("^__.*");

    if is_dynamic {
        let static_bindings = bindings_builder
            .clone()
            .generate()
            .expect("Unable to generate static bindings for shim analysis");

        let static_code = static_bindings.to_string();
        let shims = generate_shims(&static_code);

        std::fs::write(out_path.join("shims.rs"), shims).expect("Unable to write shims.rs");

        let dynamic_bindings = bindings_builder
            .dynamic_library_name("MaaFramework")
            .dynamic_link_require_all(true)
            .blocklist_item("__security_cookie")
            .raw_line("unsafe impl Send for MaaFramework {}")
            .raw_line("unsafe impl Sync for MaaFramework {}")
            .generate()
            .expect("Unable to generate dynamic bindings");

        let mut bindings_content = dynamic_bindings.to_string();

        if !bindings_content.contains(": ::libloading::Library") {
            panic!("bindgen output format changed! Cannot apply CompositeLibrary patch. Expected ': ::libloading::Library'");
        }

        bindings_content =
            bindings_content.replace(": ::libloading::Library", ": CompositeLibrary");
        bindings_content =
            bindings_content.replace("::libloading::Library::new", "CompositeLibrary::new");
        bindings_content =
            bindings_content.replace("Into<::libloading::Library>", "Into<CompositeLibrary>");

        std::fs::write(out_path.join("bindings.rs"), bindings_content)
            .expect("Couldn't write bindings!");
    } else {
        let bindings = bindings_builder
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}

#[cfg(feature = "generate-bindings")]
fn generate_shims(code: &str) -> String {
    let file = syn::parse_file(code).expect("Unable to parse bindings for shim generation");

    let mut result = String::new();

    for item in file.items {
        if let syn::Item::ForeignMod(foreign_mod) = item {
            for foreign_item in foreign_mod.items {
                if let syn::ForeignItem::Fn(func) = foreign_item {
                    if func.sig.variadic.is_some() {
                        continue;
                    }

                    let name = &func.sig.ident;
                    let inputs = func.sig.inputs.iter().map(|arg| match arg {
                        syn::FnArg::Typed(pat_type) => pat_type,
                        syn::FnArg::Receiver(_) => panic!(
                            "Unexpected receiver argument (self) in C function shim generation"
                        ),
                    });

                    let output = &func.sig.output;

                    let ret_type = match output {
                        syn::ReturnType::Default => quote::quote! { () },
                        syn::ReturnType::Type(_, ty) => quote::quote! { #ty },
                    };

                    let shim_macro = quote::quote! {
                        shim!(#name ( #(#inputs),* ) -> #ret_type);
                    };

                    use std::fmt::Write;
                    writeln!(result, "{}", shim_macro).unwrap();
                }
            }
        }
    }
    result
}

/// Copy native DLLs and other assets to target directory so `cargo run` works.
fn copy_dlls_to_target(lib_dirs: &[PathBuf], out_dir: &PathBuf) {
    // Find target/debug or target/release
    let target_dir = out_dir.ancestors().find(|p| {
        p.file_name()
            .map_or(false, |n| n == "debug" || n == "release")
            && p.parent()
                .map_or(false, |pp| pp.file_name().map_or(false, |n| n == "target"))
    });

    let target_dir = match target_dir {
        Some(dir) => dir.to_path_buf(),
        None => return,
    };

    // Find bin directory from lib directories
    let bin_dir = lib_dirs.iter().find_map(|lib| {
        let bin = if lib.ends_with("lib") {
            lib.parent().map(|p| p.join("bin"))
        } else if lib.ends_with("bin") {
            Some(lib.clone())
        } else {
            lib.parent().map(|p| p.join("bin"))
        };
        bin.filter(|b| b.exists())
    });

    let bin_dir = match bin_dir {
        Some(dir) => dir,
        None => return,
    };

    // Copy ALL files from bin directory recursively
    let copied = copy_dir_recursive(&bin_dir, &target_dir).unwrap_or(0);

    if copied > 0 {
        println!(
            "cargo:warning=Copied {} files from SDK bin/ to {}",
            copied,
            target_dir.display()
        );
    }
}

/// Helper to copy a directory recursively
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<u64> {
    if !src.exists() {
        return Ok(0);
    }

    if src.is_dir() {
        if !dst.exists() {
            std::fs::create_dir_all(dst)?;
        }
        let mut count = 0;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let dst_path = dst.join(name);
            count += copy_dir_recursive(&path, &dst_path)?;
        }
        Ok(count)
    } else {
        std::fs::copy(src, dst)?;
        Ok(1)
    }
}
