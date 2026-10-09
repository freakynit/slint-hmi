use slint::{Color, ComponentHandle, VecModel};
use std::{cell::RefCell, rc::Rc};

slint::include_modules!();

#[path = "../../hmi-common/alarms.rs"]
mod alarms;
use alarms::AlarmStore;

fn main() -> Result<(), slint::PlatformError> {
    let ui = CompactHmi::new()?;
    ui.global::<HmiTheme>().set_dark(false);
    ui.set_trends(
        Rc::new(VecModel::from(vec![TrendSeries {
            label: "Vessel".into(),
            color: Color::from_rgb_u8(0x56, 0xd9, 0xea),
            // Intentionally irregular times demonstrate real time coordinates.
            samples: Rc::new(VecModel::from(vec![
                TrendSample {
                    time: 0.0,
                    value: 24.0,
                },
                TrendSample {
                    time: 8.0,
                    value: 29.0,
                },
                TrendSample {
                    time: 19.0,
                    value: 36.0,
                },
                TrendSample {
                    time: 27.0,
                    value: 43.0,
                },
                TrendSample {
                    time: 41.0,
                    value: 42.2,
                },
                TrendSample {
                    time: 60.0,
                    value: 41.8,
                },
            ]))
            .into(),
        }]))
        .into(),
    );

    let mut store = AlarmStore::new();
    store.raise("Batch started", "Batch / B-204", AlarmSeverity::Info, 0);
    store.clear_source("Batch / B-204");
    store.raise(
        "Inspection due",
        "Vessel / V-01",
        AlarmSeverity::Warning,
        20,
    );
    store.raise(
        "Remote sensor unavailable",
        "Pressure / PT-04",
        AlarmSeverity::Critical,
        43,
    );
    let store = Rc::new(RefCell::new(store));
    ui.set_alarms(store.borrow().visible(ui.get_alarm_filter()));
    ui.set_message(
        format!(
            "{} alarms awaiting review · click the ring or drag the dial center",
            store.borrow().unacknowledged_count()
        )
        .into(),
    );

    let weak = ui.as_weak();
    ui.on_setpoint_requested(move |value| {
        if let Some(ui) = weak.upgrade() {
            // Example host-side operating limit, narrower than the dial range.
            if value <= 60.0 {
                ui.set_setpoint(value);
                ui.set_message(format!("Host accepted setpoint · {value:.1} °C").into());
            } else {
                ui.set_message("Host rejected setpoint · batch limit is 60 °C".into());
            }
        }
    });
    let weak = ui.as_weak();
    let alarms = store.clone();
    ui.on_filter_requested(move |filter| {
        if let Some(ui) = weak.upgrade() {
            ui.set_alarms(alarms.borrow().visible(filter));
        }
    });
    let weak = ui.as_weak();
    ui.on_acknowledge_requested(move |id| {
        if let Some(ui) = weak.upgrade() {
            let mut alarms = store.borrow_mut();
            alarms.acknowledge(id.as_str());
            ui.set_alarms(alarms.visible(ui.get_alarm_filter()));
            ui.set_message(format!("Acknowledged {id} · active condition preserved").into());
        }
    });
    ui.run()
}
