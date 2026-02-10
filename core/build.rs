#[cfg(windows)]
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    #[cfg(windows)]
    ensure_common_controls_v6_for_tests();

    #[cfg(windows)]
    copy_opencv_world_dll();

    built::write_built_file().expect("Failed to acquire build-time information");
}

#[cfg(windows)]
fn ensure_common_controls_v6_for_tests() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    let manifest_path = out_dir.join("actiona_ng_tests.manifest");
    let manifest_content = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" processorArchitecture="*" name="core-tests" type="win32"/>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>
"#;

    if let Err(error) = fs::write(&manifest_path, manifest_content) {
        println!(
            "cargo:warning=Failed to write Windows test manifest {}: {}",
            manifest_path.display(),
            error
        );
        return;
    }

    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest_path.display()
    );
}

#[cfg(windows)]
fn copy_opencv_world_dll() {
    for variable in [
        "DEP_OPENCV_LINK_LIBS",
        "DEP_OPENCV_LINK_PATHS",
        "OPENCV_LINK_LIBS",
        "OPENCV_LINK_PATHS",
        "OpenCV_DIR",
        "OPENCV_DIR",
        "PATH",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    let Some(profile_dir) = out_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    else {
        println!("cargo:warning=Failed to determine Cargo target profile directory");
        return;
    };

    let dll_names = find_opencv_world_dll_names(profile_dir);
    if dll_names.is_empty() {
        println!("cargo:warning=Unable to infer the OpenCV world DLL name");
        return;
    }

    let Some(source_dll) = find_opencv_world_dll_source(profile_dir, &dll_names) else {
        println!(
            "cargo:warning=Could not locate OpenCV runtime DLL (searched for: {})",
            dll_names.join(", ")
        );
        println!(
            "cargo:warning=Ensure OpenCV is installed and discoverable via OPENCV_LINK_PATHS/DEP_OPENCV_LINK_PATHS or PATH"
        );
        return;
    };

    println!("cargo:rerun-if-changed={}", source_dll.display());
    let Some(file_name) = source_dll.file_name() else {
        println!(
            "cargo:warning=Failed to determine file name for OpenCV DLL at {}",
            source_dll.display()
        );
        return;
    };

    for destination_dir in [profile_dir.to_path_buf(), profile_dir.join("deps")] {
        if let Err(error) = fs::create_dir_all(&destination_dir) {
            println!(
                "cargo:warning=Failed to create destination directory {}: {}",
                destination_dir.display(),
                error
            );
            continue;
        }

        let destination = destination_dir.join(file_name);
        if let Err(error) = fs::copy(&source_dll, &destination) {
            println!(
                "cargo:warning=Failed to copy {} to {}: {}",
                source_dll.display(),
                destination.display(),
                error
            );
        }
    }
}

#[cfg(windows)]
fn find_opencv_world_dll_names(profile_dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = BTreeSet::new();

    let mut libs = Vec::new();
    libs.extend(get_link_lib_values_from_env("DEP_OPENCV_LINK_LIBS"));
    libs.extend(get_link_lib_values_from_env("OPENCV_LINK_LIBS"));
    libs.extend(get_link_lib_values_from_opencv_build_output(profile_dir));

    for raw_lib in libs {
        let Some(lib_name) = normalize_link_lib_name(&raw_lib) else {
            continue;
        };
        if !lib_name.starts_with("opencv_world") {
            continue;
        }

        let dll_name = format!("{lib_name}.dll");
        if seen.insert(dll_name.to_ascii_lowercase()) {
            names.push(dll_name);
        }
    }

    names
}

#[cfg(windows)]
fn find_opencv_world_dll_source(profile_dir: &Path, dll_names: &[String]) -> Option<PathBuf> {
    let mut candidate_dirs = Vec::new();

    for variable in ["DEP_OPENCV_LINK_PATHS", "OPENCV_LINK_PATHS"] {
        for raw_path in get_path_values_from_env(variable) {
            push_candidate_dirs_for_link_path(&mut candidate_dirs, raw_path);
        }
    }

    for raw_path in get_link_search_paths_from_opencv_build_output(profile_dir) {
        push_candidate_dirs_for_link_path(&mut candidate_dirs, raw_path);
    }

    for variable in ["OpenCV_DIR", "OPENCV_DIR"] {
        if let Ok(raw) = env::var(variable)
            && !raw.trim().is_empty()
        {
            push_candidate_dirs_for_opencv_root(&mut candidate_dirs, PathBuf::from(raw));
        }
    }

    if let Ok(path) = env::var("PATH") {
        for path_entry in env::split_paths(&path) {
            if !path_entry.as_os_str().is_empty() {
                candidate_dirs.push(path_entry);
            }
        }
    }

    let mut seen_dirs = BTreeSet::new();
    for directory in candidate_dirs {
        if !directory.is_dir() {
            continue;
        }

        let key = directory.to_string_lossy().to_ascii_lowercase();
        if !seen_dirs.insert(key) {
            continue;
        }

        for dll_name in dll_names {
            let candidate = directory.join(dll_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(windows)]
fn push_candidate_dirs_for_link_path(candidate_dirs: &mut Vec<PathBuf>, raw_path: PathBuf) {
    if raw_path.as_os_str().is_empty() {
        return;
    }

    candidate_dirs.push(raw_path.clone());

    if let Some(file_name) = raw_path.file_name().and_then(|name| name.to_str())
        && file_name.eq_ignore_ascii_case("lib")
        && let Some(parent) = raw_path.parent()
    {
        candidate_dirs.push(parent.join("bin"));
    }
}

#[cfg(windows)]
fn push_candidate_dirs_for_opencv_root(candidate_dirs: &mut Vec<PathBuf>, root: PathBuf) {
    if root.as_os_str().is_empty() {
        return;
    }

    candidate_dirs.push(root.clone());
    candidate_dirs.push(root.join("bin"));
    candidate_dirs.push(root.join("x64").join("vc16").join("bin"));

    if let Some(parent) = root.parent() {
        candidate_dirs.push(parent.join("bin"));
    }
}

#[cfg(windows)]
fn get_link_lib_values_from_env(variable: &str) -> Vec<String> {
    let Ok(value) = env::var(variable) else {
        return Vec::new();
    };

    split_list_values(&value)
}

#[cfg(windows)]
fn get_path_values_from_env(variable: &str) -> Vec<PathBuf> {
    let Ok(value) = env::var(variable) else {
        return Vec::new();
    };

    // OpenCV env vars use `;` or `,` as separators (not spaces, which appear in paths).
    split_path_values(&value)
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

#[cfg(windows)]
fn get_link_lib_values_from_opencv_build_output(profile_dir: &Path) -> Vec<String> {
    let mut libs = Vec::new();
    for output_file in opencv_build_output_files(profile_dir) {
        let Ok(contents) = fs::read_to_string(output_file) else {
            continue;
        };
        for line in contents.lines() {
            if let Some(value) = line
                .strip_prefix("cargo::rustc-link-lib=")
                .or_else(|| line.strip_prefix("cargo:rustc-link-lib="))
            {
                libs.push(value.trim().to_string());
            }
        }
    }

    libs
}

#[cfg(windows)]
fn get_link_search_paths_from_opencv_build_output(profile_dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for output_file in opencv_build_output_files(profile_dir) {
        let Ok(contents) = fs::read_to_string(output_file) else {
            continue;
        };
        for line in contents.lines() {
            let Some(raw_value) = line
                .strip_prefix("cargo::rustc-link-search=")
                .or_else(|| line.strip_prefix("cargo:rustc-link-search="))
            else {
                continue;
            };

            let value = raw_value
                .strip_prefix("native=")
                .unwrap_or(raw_value)
                .trim();
            if value.is_empty() {
                continue;
            }
            paths.push(PathBuf::from(value));
        }
    }

    paths
}

#[cfg(windows)]
fn opencv_build_output_files(profile_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let build_dir = profile_dir.join("build");
    let Ok(entries) = fs::read_dir(build_dir) else {
        return files;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("opencv-") {
            continue;
        }

        let output_file = entry.path().join("output");
        if output_file.is_file() {
            files.push(output_file);
        }
    }

    files
}

#[cfg(windows)]
fn normalize_link_lib_name(raw: &str) -> Option<String> {
    let mut lib = raw.trim();
    if lib.is_empty() {
        return None;
    }

    for prefix in ["dylib=", "static=", "framework="] {
        if let Some(stripped) = lib.strip_prefix(prefix) {
            lib = stripped;
            break;
        }
    }

    if lib.is_empty() {
        None
    } else {
        Some(lib.to_string())
    }
}

#[cfg(windows)]
fn split_list_values(value: &str) -> Vec<String> {
    value
        .split([';', ',', ' ', '\n', '\r', '\t'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(windows)]
fn split_path_values(value: &str) -> Vec<String> {
    value
        .split([';', ',', '\n', '\r', '\t'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}
