use hbb_common::{bail, ResultType};

pub(crate) const PUBLIC_UPDATE_PACKAGE_ID: &str = "rustdesk";

#[derive(Clone, Copy)]
pub(crate) struct UpdateArtifactQuery<'a> {
    pub platform: &'a str,
    pub arch: &'a str,
    pub format: &'a str,
}

#[derive(Clone, Copy)]
pub(crate) struct UpdateMetadataRequirements<'a> {
    pub expected_version: &'a str,
    pub expected_release_id: &'a str,
    pub expected_artifact_url_prefix: &'a str,
    pub artifact: UpdateArtifactQuery<'a>,
}

#[derive(Clone)]
pub(crate) struct VerifiedUpdateArtifact {
    pub version: String,
    pub url: String,
    #[cfg(target_os = "macos")]
    pub file_name: String,
    pub size: u64,
    pub sha256: String,
}

pub(crate) fn verify_update_metadata(
    _metadata_bytes: &[u8],
    _signature_bytes: &[u8],
    _requirements: UpdateMetadataRequirements<'_>,
) -> ResultType<VerifiedUpdateArtifact> {
    bail!("update metadata verification is not supported in lan-only builds")
}

#[cfg(target_os = "macos")]
pub(crate) use offline::{verify_offline_update_metadata, OfflineUpdateMetadataRequirements};

#[cfg(target_os = "macos")]
mod offline {
    use super::*;

    pub(crate) struct OfflineUpdateMetadataRequirements<'a> {
        pub local_file_name: &'a str,
        pub expected_package_id: &'a str,
        pub artifact: UpdateArtifactQuery<'a>,
    }

    pub(crate) fn verify_offline_update_metadata(
        _metadata_bytes: &[u8],
        _signature_bytes: &[u8],
        _requirements: OfflineUpdateMetadataRequirements<'_>,
    ) -> ResultType<VerifiedUpdateArtifact> {
        bail!("offline update verification is not supported in lan-only builds")
    }
}
