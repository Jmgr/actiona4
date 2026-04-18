use std::{path::Path, process::Command};

use color_eyre::{Result, eyre::eyre};

use crate::util::run_command;

pub fn lint_e2e_typescript(workspace_root: &Path) -> Result<()> {
    let e2e_dir = workspace_root.join("crates/e2e");
    let tsc_path = if cfg!(windows) {
        e2e_dir.join("node_modules/.bin/tsc.cmd")
    } else {
        e2e_dir.join("node_modules/.bin/tsc")
    };

    if !tsc_path.exists() {
        return Err(eyre!(
            "TypeScript compiler not found at {}. Run `npm ci --prefix crates/e2e` first.",
            tsc_path.display()
        ));
    }

    run_command(
        Command::new(&tsc_path)
            .arg("-p")
            .arg(e2e_dir.join("scripts/tsconfig.json"))
            .arg("--noEmit")
            .current_dir(workspace_root),
        "TypeScript type-checking failed for crates/e2e/scripts.",
    )
}
