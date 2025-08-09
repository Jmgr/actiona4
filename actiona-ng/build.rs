use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }
}
