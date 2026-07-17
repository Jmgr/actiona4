use std::{
    collections::HashMap,
    sync::{
        OnceLock,
        atomic::{AtomicI64, Ordering},
    },
};

use parking_lot::Mutex;
use tokio::sync::broadcast;

const TEST_ACTION_VISIT_CHANNEL_CAPACITY: usize = 1024;

static NEXT_TEST_ACTION_VISIT_ID: AtomicI64 = AtomicI64::new(1);
static TEST_ACTION_VISIT_LABELS: OnceLock<Mutex<HashMap<i64, &'static str>>> = OnceLock::new();
static TEST_ACTION_VISITS: OnceLock<broadcast::Sender<i64>> = OnceLock::new();

fn visit_labels() -> &'static Mutex<HashMap<i64, &'static str>> {
    TEST_ACTION_VISIT_LABELS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn visit_sender() -> &'static broadcast::Sender<i64> {
    TEST_ACTION_VISITS.get_or_init(|| {
        let (sender, _) = broadcast::channel(TEST_ACTION_VISIT_CHANNEL_CAPACITY);
        sender
    })
}

pub fn register_test_action_visit_label(label: &'static str) -> i64 {
    let id = NEXT_TEST_ACTION_VISIT_ID.fetch_add(1, Ordering::Relaxed);
    visit_labels().lock().insert(id, label);
    id
}

pub fn test_action_visit_label(id: i64) -> Option<&'static str> {
    visit_labels().lock().get(&id).copied()
}

pub fn subscribe_test_action_visits() -> broadcast::Receiver<i64> {
    visit_sender().subscribe()
}

pub fn record_test_action_visit(id: i64) {
    _ = visit_sender().send(id);
}
