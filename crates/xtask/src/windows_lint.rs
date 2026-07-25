//! Cross-compiles the workspace for Windows from Linux so that clippy sees the
//! Windows-only code. Nothing is linked or run: `cargo xwin` supplies the MSVC CRT and
//! Windows SDK headers, and the opencv crate is pointed at the prebuilt Windows package.

use std::{
    env,
    fmt::Write as _,
    fs,
    fs::File,
    io::Write,
    iter,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use color_eyre::{Result, eyre::eyre};
use sha2::{Digest, Sha256};

use crate::util::run_command;

const TARGET: &str = "x86_64-pc-windows-msvc";
const OPENCV_VERSION: &str = "4.12.0";
const OPENCV_SHA256: &str = "b753b14d880b9bc8d89d6acd3b665c040baec0211078435432fcae117db707af";
const OPENCV_URL_BASE: &str = "https://github.com/opencv/opencv/releases/download";
/// The Microsoft STL bundled with recent Windows SDKs rejects older Clang with a
/// `static_assert`, so an older compiler cannot build the OpenCV bindings at all.
const MINIMUM_CLANG_MAJOR: u32 = 19;
/// Upper bound for the `clang-<major>` binaries to look for on PATH.
const NEWEST_KNOWN_CLANG_MAJOR: u32 = 30;
/// Overrides where the Windows SDK and OpenCV headers are kept, for CI, which caches them
/// separately from the target directory.
const CROSS_DIR_VAR: &str = "ACTIONA_WINDOWS_CROSS_DIR";

pub async fn lint_windows(workspace_root: &Path) -> Result<()> {
    let cross_dir = env::var_os(CROSS_DIR_VAR).map_or_else(
        || workspace_root.join("target").join("windows-cross"),
        PathBuf::from,
    );
    fs::create_dir_all(&cross_dir)?;

    ensure_cargo_xwin()?;
    ensure_rust_target()?;
    ensure_llvm_tools()?;

    let clang_dir = stage_clang(&cross_dir)?;
    let opencv_dir = ensure_opencv(&cross_dir).await?;

    run_command(
        Command::new("cargo")
            .args([
                "xwin",
                "clippy",
                "--all",
                "--all-targets",
                "--all-features",
                "--target",
                TARGET,
            ])
            .current_dir(workspace_root)
            .env("PATH", prepend_to_path(&clang_dir)?)
            .env("XWIN_CACHE_DIR", cross_dir.join("xwin"))
            // The prebuilt package is neither a pkg-config nor a vcpkg installation, so the
            // opencv crate has to be pointed at it by hand. Only the headers are actually
            // used: the import library is named but never linked, since this only lints.
            .env("OPENCV_INCLUDE_PATHS", opencv_dir.join("include"))
            .env(
                "OPENCV_LINK_PATHS",
                opencv_dir.join("x64").join("vc16").join("lib"),
            )
            .env("OPENCV_LINK_LIBS", opencv_world_library())
            // Matches -C target-feature=+crt-static from Makefile.toml.
            .env("OPENCV_MSVC_CRT", "static"),
        "Windows cross-lint failed.",
    )
}

fn ensure_cargo_xwin() -> Result<()> {
    let installed = Command::new("cargo")
        .args(["xwin", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());

    if installed {
        return Ok(());
    }

    println!("Installing cargo-xwin...");

    run_command(
        Command::new("cargo").args(["install", "--locked", "cargo-xwin"]),
        "Failed to install cargo-xwin.",
    )
}

fn ensure_rust_target() -> Result<()> {
    run_command(
        Command::new("rustup").args(["target", "add", TARGET]),
        format!("Failed to install the {TARGET} Rust target.").as_str(),
    )
}

/// cc-rs archives the OpenCV bindings with `llvm-lib`, which cargo-xwin substitutes with the
/// `llvm-ar` from the toolchain when `llvm-lib` is not on PATH. Install the component that
/// provides it only when neither is there: rustup refuses to install it over a distribution's
/// own copy of those binaries, and that copy works just as well.
fn ensure_llvm_tools() -> Result<()> {
    let installed = which("llvm-lib").is_some()
        || toolchain_bin_dir().is_some_and(|dir| dir.join("llvm-ar").exists());

    if installed {
        return Ok(());
    }

    run_command(
        Command::new("rustup").args(["component", "add", "llvm-tools"]),
        "Failed to install the llvm-tools component, which provides the archiver the OpenCV \
         bindings are built with. Installing LLVM system-wide works too (`sudo apt install \
         llvm`).",
    )
}

/// The toolchain directory cargo-xwin looks in for the LLVM tools.
fn toolchain_bin_dir() -> Option<PathBuf> {
    let output = Command::new("rustc")
        .args(["--print", "target-libdir"])
        .output()
        .ok()?;
    let target_libdir = String::from_utf8(output.stdout).ok()?;

    Some(Path::new(target_libdir.trim()).parent()?.join("bin"))
}

/// Symlinks a recent enough Clang into the cross directory and returns that directory, to be
/// prepended to PATH: cargo-xwin builds its `clang-cl` shim out of the first `clang` it finds
/// on PATH, which on Debian and Ubuntu is usually older than the Windows SDK accepts.
fn stage_clang(cross_dir: &Path) -> Result<PathBuf> {
    let clang = find_clang()?;
    let bin_dir = cross_dir.join("bin");
    let staged = bin_dir.join("clang");

    fs::create_dir_all(&bin_dir)?;

    if staged.is_symlink() || staged.exists() {
        fs::remove_file(&staged)?;
    }

    symlink(&clang, &staged)?;

    // cargo-xwin points its own shim at whichever clang it found on PATH the first time it
    // ran, and only replaces that symlink when the symlink still resolves. Drop it while it
    // dangles — after the cross directory has been moved, say — so that it gets recreated.
    let shim = cross_dir.join("xwin").join("clang-cl");
    if shim.is_symlink() && !shim.exists() {
        fs::remove_file(&shim)?;
    }

    Ok(bin_dir)
}

fn find_clang() -> Result<PathBuf> {
    let candidates = iter::once("clang".to_owned()).chain(
        (MINIMUM_CLANG_MAJOR..=NEWEST_KNOWN_CLANG_MAJOR)
            .rev()
            .map(|major| format!("clang-{major}")),
    );

    for candidate in candidates {
        let Some(path) = which(&candidate) else {
            continue;
        };

        if clang_major_version(&path).is_some_and(|major| major >= MINIMUM_CLANG_MAJOR) {
            return Ok(path);
        }
    }

    Err(eyre!(
        "No Clang {MINIMUM_CLANG_MAJOR} or newer found on PATH. The Microsoft STL shipped \
         with the Windows SDK does not compile with anything older; install one with \
         `sudo apt install clang-{MINIMUM_CLANG_MAJOR}`."
    ))
}

/// Reads the major version out of `clang --version`, whose first line looks like
/// `Ubuntu clang version 19.1.1 (1ubuntu1)`.
fn clang_major_version(clang: &Path) -> Option<u32> {
    let output = Command::new(clang).arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;

    version
        .split("clang version ")
        .nth(1)?
        .split(['.', '-', ' ', '\n'])
        .next()?
        .parse()
        .ok()
}

async fn ensure_opencv(cross_dir: &Path) -> Result<PathBuf> {
    let build_dir = cross_dir
        .join(format!("opencv-{OPENCV_VERSION}"))
        .join("opencv")
        .join("build");

    if build_dir.join("include").exists() {
        return Ok(build_dir);
    }

    let seven_zip = which("7z").ok_or_else(|| {
        eyre!(
            "7z not found on PATH. The Windows OpenCV package is a 7-Zip archive; install it \
             with `sudo apt install 7zip`."
        )
    })?;

    let archive_path = cross_dir.join(format!("opencv-{OPENCV_VERSION}-windows.exe"));
    download_opencv(&archive_path).await?;

    // Only the headers and the import library are needed: leaving out the DLLs and the Java
    // and Python bindings turns 900 MB of extracted files into 15 MB.
    let install_dir = cross_dir.join(format!("opencv-{OPENCV_VERSION}"));
    run_command(
        Command::new(seven_zip)
            .arg("x")
            .arg("-y")
            .arg("-bso0")
            .arg("-bsp0")
            .arg(&archive_path)
            .arg(format!("-o{}", install_dir.display()))
            .arg("opencv/build/include")
            .arg("opencv/build/x64/vc16/lib")
            .arg("-r"),
        "Failed to extract the Windows OpenCV package.",
    )?;

    fs::remove_file(&archive_path)?;

    Ok(build_dir)
}

async fn download_opencv(destination: &Path) -> Result<()> {
    let url = format!("{OPENCV_URL_BASE}/{OPENCV_VERSION}/opencv-{OPENCV_VERSION}-windows.exe");
    eprintln!("Downloading OpenCV {OPENCV_VERSION} for Windows from {url}...");

    let mut response = reqwest::get(&url)
        .await
        .map_err(|error| eyre!("Failed to download {url}: {error}"))?
        .error_for_status()
        .map_err(|error| eyre!("Download failed for {url}: {error}"))?;

    let mut file = File::create(destination)?;
    let mut hasher = Sha256::new();

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| eyre!("Failed to read download response for {url}: {error}"))?
    {
        hasher.update(&chunk);
        file.write_all(&chunk)?;
    }

    file.flush()?;

    let mut digest = String::new();
    for byte in hasher.finalize() {
        write!(digest, "{byte:02x}")?;
    }

    if digest != OPENCV_SHA256 {
        fs::remove_file(destination)?;

        return Err(eyre!(
            "Checksum mismatch for {url}: expected {OPENCV_SHA256}, got {digest}."
        ));
    }

    Ok(())
}

/// Name of the OpenCV import library, `opencv_world4120` for OpenCV 4.12.0.
fn opencv_world_library() -> String {
    let digits: String = OPENCV_VERSION.split('.').collect();

    format!("opencv_world{digits}")
}

fn which(name: &str) -> Option<PathBuf> {
    let output = Command::new("which").arg(name).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;

    Some(PathBuf::from(path.trim()))
}

fn prepend_to_path(dir: &Path) -> Result<String> {
    let mut paths = vec![dir.to_path_buf()];
    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }

    env::join_paths(paths)
        .map_err(|error| eyre!("Failed to construct PATH for the Windows cross-lint: {error}"))
        .map(|value| value.to_string_lossy().into_owned())
}
