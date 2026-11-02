use hbb_common::{bail, ResultType};

pub(crate) fn execute_macos_update(_args: &[String]) -> ResultType<()> {
    bail!("software update is not supported in lan-only builds")
}
