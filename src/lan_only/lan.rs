use hbb_common::ResultType;

#[cfg(not(target_os = "ios"))]
pub(super) fn start_listening() -> ResultType<()> {
    Ok(())
}

/// Moved here from the real `lan` module so the signature the callers see stays
/// identical. The real one carries `#[tokio::main]`, so it reads as a plain
/// blocking call at every call site; keep that shape.
pub fn discover() -> ResultType<()> {
    Ok(())
}

pub fn send_wol(_id: String) {}
