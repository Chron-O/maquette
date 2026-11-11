// The bubbles of the three header indicators (prototype 06 and 07): the readers of this device,
// the role and the linked devices, the public screens.

use crate::readers::info;
use crate::live::last_reading;
use crate::test_data::{
    CURRENT_EVENT, DEVICE_ROLES, DeviceKind, Generated, LinkedDevice, PublicScreen, READER_USAGES, SCREEN_CONTENTS,
    format_clock, format_clock_seconds,
};
use crate::{App, Equipment, EquipmentReader, LinkedDeviceRow, PublicScreenRow, ScreenPreview, model};
use slint::SharedString;
use slint::ComponentHandle;

pub fn show(window: &App, data: Option<&Generated>) {
    let equipment = window.global::<Equipment>();
    equipment.set_usages(model(READER_USAGES.iter().map(|&u| SharedString::from(u)).collect()));
    equipment.set_contents(model(SCREEN_CONTENTS.iter().map(|&c| SharedString::from(c)).collect()));
    equipment.set_roles(model(DEVICE_ROLES.iter().map(|&r| SharedString::from(r)).collect()));
    equipment.set_this_role("Rôle GEC (toute la course)".into());
    let Some(data) = data else {
        equipment.set_readers(model(Vec::new()));
        equipment.set_courses(model(Vec::new()));
        equipment.set_devices(model(Vec::new()));
        equipment.set_has_request(false);
        equipment.set_course_state("Course hors ligne".into());
        equipment.set_screens(model(Vec::new()));
        return;
    };
    show_devices(&equipment, data);
    show_screens(&equipment, data);
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

fn screen_row(data: &Generated, screen: &PublicScreen) -> PublicScreenRow {
    let reader = |side: usize| screen.readers[side].and_then(|r| data.readers.get(r)).map_or("Aucun".into(), |r| r.name.as_str().into());
    let index = |side: usize| screen.readers[side].map_or(-1, |r| r as i32);
    PublicScreenRow {
        name: screen.name.as_str().into(),
        detail: match screen.size {
            Some(size) => format!("{size} · plein écran").into(),
            None => "fenêtre libre".into(),
        },
        window: screen.size.is_none(),
        on: screen.on,
        content: screen.content as i32,
        left: reader(0),
        right: reader(1),
        left_index: index(0),
        right_index: index(1),
        provisional: screen.provisional,
        names: screen.names,
        beeps: screen.beeps,
    }
}

// The screens, and what they show now: the card just read, the time, the next starters.
fn show_screens(equipment: &Equipment, data: &Generated) {
    equipment.set_screens(model(data.screens.iter().map(|s| screen_row(data, s)).collect()));
    let event = &data.events[CURRENT_EVENT];
    let clock = data.clock();
    let last = last_reading(event, &data.readers, clock, false);
    let reader = event.readings.last().map_or(-1, |r| r.reader as i32);
    let mut next: Vec<_> = event.runners.iter().filter(|r| !r.started && r.start >= clock).collect();
    next.sort_by_key(|r| r.start);
    equipment.set_start_point(format!("S{}", event.courses.first().map_or(1, |c| c.start_point)).into());
    equipment.set_preview(ScreenPreview {
        reading: last.live,
        reader,
        name: last.name,
        category: last.category,
        time: last.time,
        place: last.place,
        clock: format_clock_seconds(clock).into(),
        starters: model(next.iter().take(3).map(|r| format!("{} · {}", format_clock(r.start), r.name()).into()).collect()),
    });
}
