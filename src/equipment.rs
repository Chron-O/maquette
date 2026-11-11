// The bubbles of the three header indicators (prototype 06 and 07): the readers of this device,
// the role and the linked devices, the public screens.

use crate::readers::info;
use crate::test_data::{DEVICE_ROLES, DeviceKind, Generated, LinkedDevice, READER_USAGES};
use crate::{App, Equipment, EquipmentReader, LinkedDeviceRow, model};
use slint::SharedString;
use slint::ComponentHandle;

pub fn show(window: &App, data: Option<&Generated>) {
    let equipment = window.global::<Equipment>();
    equipment.set_usages(model(READER_USAGES.iter().map(|&u| SharedString::from(u)).collect()));
    equipment.set_roles(model(DEVICE_ROLES.iter().map(|&r| SharedString::from(r)).collect()));
    equipment.set_this_role("Rôle GEC (toute la course)".into());
    let Some(data) = data else {
        equipment.set_readers(model(Vec::new()));
        equipment.set_courses(model(Vec::new()));
        equipment.set_devices(model(Vec::new()));
        equipment.set_has_request(false);
        equipment.set_course_state("Course hors ligne".into());
        return;
    };
    show_devices(&equipment, data);
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

fn kind(kind: DeviceKind) -> i32 {
    match kind {
        DeviceKind::Laptop => 0,
        DeviceKind::Phone => 1,
        DeviceKind::Tablet => 2,
    }
}

fn device_row(device: &LinkedDevice) -> LinkedDeviceRow {
    LinkedDeviceRow {
        kind: kind(device.kind),
        name: device.name.as_str().into(),
        role: DEVICE_ROLES[device.role].into(),
        role_choice: device.role as i32,
        offline: device.waiting.is_some(),
        state: match device.waiting {
            Some(waiting) => format!("Hors ligne · {waiting} en attente").into(),
            None => "Synchronisé".into(),
        },
    }
}

// This device, the linked ones, and the one asking to join.
fn show_devices(equipment: &Equipment, data: &Generated) {
    let linked = data.devices.len() + 1;
    equipment.set_course_state(format!("Course en ligne · {linked} appareils").into());
    equipment.set_devices(model(data.devices.iter().map(device_row).collect()));
    equipment.set_has_request(data.request.is_some());
    if let Some(request) = &data.request {
        equipment.set_request(LinkedDeviceRow {
            kind: kind(request.kind),
            name: request.name.as_str().into(),
            role: DEVICE_ROLES[request.role].into(),
            role_choice: request.role as i32,
            offline: false,
            state: SharedString::new(),
        });
        equipment.set_request_word(request.word.into());
    }
}
