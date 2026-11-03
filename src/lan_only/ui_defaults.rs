//! Seeds the settings the Flutter UI already reads, so a lan-only build stops
//! offering what the feature drops from Rust: account auth, the network
//! (ID/relay/API) tab, update cards and the ID board.
//!
//! Called from `flutter_ffi::initialize` before an operator's custom client config
//! is loaded, and every key is seeded with `or_insert` so `custom.txt` still wins.

// The only caller sits behind the `flutter` feature, and the lan-only CI check builds
// without it, so outside a Flutter build everything here is dead code there.
#![cfg_attr(not(feature = "flutter"), allow(dead_code))]

use hbb_common::config::{BUILTIN_SETTINGS, HARD_SETTINGS};

pub(crate) fn apply() {
    seed(&HARD_SETTINGS, &[("disable-account", "Y")]);
    seed(
        &BUILTIN_SETTINGS,
        &[
            ("hide-network-settings", "Y"),
            ("hide-help-cards", "Y"),
            // not in KEYS_BUILDIN_SETTINGS, so only this build reads it
            ("hide-id-board", "Y"),
            ("hide-p2p-settings", "Y"),
            ("hide-update-settings", "Y"),
        ],
    );
}

fn seed(
    map: &std::sync::RwLock<std::collections::HashMap<String, String>>,
    defaults: &[(&str, &str)],
) {
    // A poisoned lock means another thread panicked mid-write; leaving the map as
    // it is stays on the safe side instead of unwrapping.
    let Ok(mut map) = map.write() else {
        return;
    };
    for (key, value) in defaults {
        map.entry((*key).to_owned())
            .or_insert_with(|| (*value).to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // On a local map: the real statics are process-wide and shared with the
    // config tests, so seeding them here would leak into unrelated cases.
    #[test]
    fn seeds_without_overriding_existing_values() {
        let map = std::sync::RwLock::new(std::collections::HashMap::from([(
            "disable-account".to_owned(),
            "N".to_owned(),
        )]));
        seed(&map, &[("disable-account", "Y"), ("hide-id-board", "Y")]);
        let map = map.read().unwrap();
        assert_eq!(map.get("disable-account").map(String::as_str), Some("N"));
        assert_eq!(map.get("hide-id-board").map(String::as_str), Some("Y"));
    }
}
