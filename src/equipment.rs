// The bubbles of the three header indicators (prototype 06 and 07): the readers of this device,
// the role and the linked devices, the public screens.

use crate::readers::info;
use crate::test_data::{Generated, READER_USAGES};
use crate::{App, Equipment, EquipmentReader, model};
use slint::SharedString;
use slint::ComponentHandle;

pub fn show(window: &App, data: Option<&Generated>) {
    let equipment = window.global::<Equipment>();
    equipment.set_usages(model(READER_USAGES.iter().map(|&u| SharedString::from(u)).collect()));
    let Some(data) = data else {
        equipment.set_readers(model(Vec::new()));
        equipment.set_courses(model(Vec::new()));
        return;
    };
    let courses = data.reader_courses();
    equipment.set_courses(model(courses.iter().map(|c| SharedString::from(c.as_str())).collect()));
    equipment.set_readers(model(
        data.readers
            .iter()
            .enumerate()
            .map(|(i, reader)| EquipmentReader {
                info: info(data, i),
                course: match reader.usage {
                    Some(_) => courses.get(reader.course).map_or("aucune course".into(), |c| c.as_str().into()),
                    None => "aucune course".into(),
                },
                usage: reader.usage.map_or(-1, |u| u as i32),
                course_choice: if reader.usage.is_some() { reader.course as i32 } else { -1 },
            })
            .collect(),
    ));
}
