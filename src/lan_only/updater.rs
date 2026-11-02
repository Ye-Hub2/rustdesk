use hbb_common::ResultType;
use std::path::{Path, PathBuf};

use crate::update_metadata::{UpdateArtifactQuery, VerifiedUpdateArtifact};

pub use artifact::{get_update_download_file_from_url, current_update_artifact_query};

mod artifact {
    use super::*;

    pub fn current_update_artifact_query(_update_msi: bool) -> UpdateArtifactQuery<'static> {
        UpdateArtifactQuery {
            platform: "",
            arch: "",
            format: "",
        }
    }

    pub fn get_update_download_file_from_url(_url: &str) -> Option<PathBuf> {
        None
    }
}

pub fn verified_update_artifact_for_release_page_url(
    _release_page_url: &str,
    _query: UpdateArtifactQuery<'_>,
) -> ResultType<VerifiedUpdateArtifact> {
    unreachable_update()
}

pub fn verified_update_artifact_for_download_url(
    _download_url: &str,
) -> ResultType<VerifiedUpdateArtifact> {
    unreachable_update()
}

fn unreachable_update<T>() -> ResultType<T> {
    Err(hbb_common::anyhow::anyhow!(
        "software update is not supported in lan-only builds"
    ))
}

pub(crate) fn remove_update_file(_file_path: &Path) {}

const CONTROLLING_SESSION_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

pub fn update_controlling_session_count(count: usize) {
    CONTROLLING_SESSION_COUNT.store(count, std::sync::atomic::Ordering::SeqCst);
}

pub fn start_auto_update() {}

pub fn manually_check_update() -> ResultType<()> {
    Ok(())
}

pub fn stop_auto_update() {}

pub fn has_no_active_conns() -> bool {
    crate::Connection::alive_conns().is_empty() && has_no_controlling_conns()
}

pub fn has_no_active_conns_ipc() -> bool {
    has_no_active_conns()
}

pub fn start_auto_update_macos() {}

pub fn check_update_as_root() -> ResultType<bool> {
    Ok(false)
}

fn has_no_controlling_conns() -> bool {
    CONTROLLING_SESSION_COUNT.load(std::sync::atomic::Ordering::SeqCst) == 0
}
