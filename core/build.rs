#[cfg(windows)]
use std::{env, fs, path::PathBuf};

use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    #[cfg(windows)]
    ensure_common_controls_v6_for_tests();

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
