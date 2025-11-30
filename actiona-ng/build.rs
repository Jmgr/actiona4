use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    built::write_built_file().expect("Failed to acquire build-time information");
}
