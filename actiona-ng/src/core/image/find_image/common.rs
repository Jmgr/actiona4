use std::thread::available_parallelism;

/// Choose a conservative worker count to avoid saturating the machine.
///
/// The intent is to keep image matching responsive without starving
/// the rest of the system.
///
/// Minimum is 1, maximum is half of the available hardware threads.
pub fn ideal_thread_count() -> usize {
    let available = available_parallelism().map(|n| n.get()).unwrap_or(1);
    (available / 2).max(1)
}
