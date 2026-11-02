use hbb_common::lazy_static::lazy_static;
use hbb_common::tokio::sync::broadcast;
use std::sync::Mutex;

// The sender is never used: `signal_receiver` hands out receivers that the
// session select loops wait on, and they have to stay pending instead of
// resolving.
lazy_static! {
    static ref SENDER: Mutex<broadcast::Sender<Vec<i32>>> = Mutex::new(broadcast::channel(16).0);
}

pub fn start() {}

#[cfg(not(any(target_os = "ios")))]
pub fn signal_receiver() -> broadcast::Receiver<Vec<i32>> {
    match SENDER.lock() {
        Ok(sender) => sender.subscribe(),
        Err(_) => broadcast::channel(16).1,
    }
}

#[allow(unused)]
#[cfg(not(any(target_os = "ios")))]
pub fn is_pro() -> bool {
    false
}

#[cfg(feature = "flutter")]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn register_switch_grant(_switch_uuid: String) {}
