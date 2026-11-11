// The readers plugged into this device, for every reader tile (prototype: tile, RD).

use crate::test_data::{ENTRY_READER, FINISH_READERS, FREE_READER, Generated, ReaderStatus};
use crate::{App, ReaderInfo, ReaderMode, Readers, model};
use slint::ComponentHandle;

pub fn info(data: &Generated, index: usize) -> ReaderInfo {
    let reader = &data.readers[index];
    ReaderInfo {
        index: index as i32,
        name: reader.name.as_str().into(),
        serial: format!("n°\u{a0}{}", reader.serial).into(),   // « n° » never alone at a line end
        pinned: reader.pinned,
        mode: match data.reader_status(index) {
            ReaderStatus::Ready => ReaderMode::Ready,
            ReaderStatus::Reading => ReaderMode::Reading,
            ReaderStatus::Unplugged => ReaderMode::Unplugged,
        },
    }
}

pub fn show(window: &App, data: Option<&Generated>) {
    let readers = window.global::<Readers>();
    let Some(data) = data else {
        readers.set_finish(model(Vec::new()));
        readers.set_entry(ReaderInfo::default());
        readers.set_free(ReaderInfo::default());
        return;
    };
    readers.set_finish(model((0..FINISH_READERS).map(|i| info(data, i)).collect()));
    readers.set_entry(info(data, ENTRY_READER));
    readers.set_free(info(data, FREE_READER));
}
