use scrap::record::RecordState;
use std::sync::mpsc::Receiver;

pub fn is_enable() -> bool {
    false
}

pub fn run(_rx: Receiver<RecordState>) {}
