use actiona_ng::{core::ui::Ui, runtime::Runtime};
use libtest_mimic_collect::libtest_mimic::Arguments;

#[macro_use]
extern crate libtest_mimic_collect;

#[test]
fn test_success() {
    Runtime::test_with_ui(|runtime, _| async {
        let ui = Ui::new(runtime);
        let button = ui.display_messagebox().await.unwrap();
        println!("button: {button:?}");
    });
}

pub fn main() {
    let mut args = Arguments::from_args();
    args.test_threads = Some(1);
    libtest_mimic_collect::TestCollection::run_with_args(args);
}
