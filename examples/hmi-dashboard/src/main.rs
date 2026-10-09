use std::{cell::RefCell, rc::Rc, time::Duration};

use slint::{Color, ComponentHandle, Model, VecModel};

slint::include_modules!();

#[path = "../../hmi-common/alarms.rs"]
mod alarms;
use alarms::AlarmStore;

const HISTORY: usize = 121;

struct Simulation {
    seconds: u64,
    temperature: f32,
    pressure_high: bool,
    alarms: AlarmStore,
}

fn append(model: &VecModel<TrendSample>, sample: TrendSample) {
    if model.row_count() >= HISTORY {
        model.remove(0);
    }
    model.push(sample);
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = HmiDashboard::new()?;
    let temperature = Rc::new(VecModel::<TrendSample>::default());
    let target = Rc::new(VecModel::<TrendSample>::default());
    let pressure_history = Rc::new(VecModel::<TrendSample>::default());
    // Prefill two minutes so the example also demonstrates a fully populated trend.
    for second in 0..HISTORY {
        temperature.push(TrendSample {
            time: second as f32,
            value: 65.0 + 4.0 * (second as f32 / 13.0).sin(),
        });
        target.push(TrendSample {
            time: second as f32,
            value: 65.0,
        });
        pressure_history.push(TrendSample {
            time: second as f32,
            value: 62.0 + 7.0 * (second as f32 / 11.0).sin(),
        });
    }
    ui.set_temperature(temperature.row_data(HISTORY - 1).unwrap().value);
    ui.set_pressure(pressure_history.row_data(HISTORY - 1).unwrap().value);
    ui.set_trends(
        Rc::new(VecModel::from(vec![
            TrendSeries {
                label: "Supply".into(),
                color: Color::from_rgb_u8(0x56, 0xd9, 0xea),
                samples: temperature.clone().into(),
            },
            TrendSeries {
                label: "Setpoint".into(),
                color: Color::from_rgb_u8(0xdf, 0xae, 0xff),
                samples: target.clone().into(),
            },
            TrendSeries {
                label: "Pressure".into(),
                color: Color::from_rgb_u8(0x14, 0x72, 0x4e),
                samples: pressure_history.clone().into(),
            },
        ]))
        .into(),
    );

    let mut store = AlarmStore::new();
    store.raise("Controller online", "PLC-01", AlarmSeverity::Info, 0);
    store.raise(
        "Startup pressure transient",
        "Startup",
        AlarmSeverity::Warning,
        10,
    );
    store.clear_source("Startup");
    store.raise(
        "Inspection due",
        "Filter / F-02",
        AlarmSeverity::Warning,
        80,
    );
    let simulation = Rc::new(RefCell::new(Simulation {
        seconds: 120,
        temperature: ui.get_temperature(),
        pressure_high: false,
        alarms: store,
    }));
    ui.set_alarms(simulation.borrow().alarms.visible(ui.get_alarm_filter()));
    ui.set_unacknowledged(simulation.borrow().alarms.unacknowledged_count());

    let weak = ui.as_weak();
    let state = simulation.clone();
    ui.on_filter_requested(move |filter| {
        if let Some(ui) = weak.upgrade() {
            ui.set_alarms(state.borrow().alarms.visible(filter));
        }
    });
    let weak = ui.as_weak();
    let state = simulation.clone();
    ui.on_acknowledge_requested(move |id| {
        if let Some(ui) = weak.upgrade() {
            let mut state = state.borrow_mut();
            state.alarms.acknowledge(id.as_str());
            ui.set_alarms(state.alarms.visible(ui.get_alarm_filter()));
            ui.set_unacknowledged(state.alarms.unacknowledged_count());
            ui.set_status(format!("Acknowledged {id} · host model updated").into());
        }
    });
    let weak = ui.as_weak();
    let state = simulation.clone();
    ui.on_fault_changed(move |fault| {
        if let Some(ui) = weak.upgrade() {
            let mut state = state.borrow_mut();
            if fault {
                let seconds = state.seconds;
                state.alarms.raise(
                    "Pump interlock tripped",
                    "Pump / P-01",
                    AlarmSeverity::Critical,
                    seconds,
                );
            } else {
                state.alarms.clear_source("Pump / P-01");
            }
            ui.set_alarms(state.alarms.visible(ui.get_alarm_filter()));
            ui.set_unacknowledged(state.alarms.unacknowledged_count());
            ui.set_status(
                if fault {
                    "Fault injected · setpoint locked"
                } else {
                    "Fault cleared · alarm retained in history"
                }
                .into(),
            );
        }
    });
    let weak = ui.as_weak();
    ui.on_setpoint_changed(move |value| {
        if let Some(ui) = weak.upgrade() {
            ui.set_status(format!("Setpoint accepted · {value:.1} °C").into());
        }
    });
    let weak = ui.as_weak();
    ui.on_pump_speed_changed(move |value| {
        if let Some(ui) = weak.upgrade() {
            ui.set_status(format!("Pump speed accepted · {value:.0} %").into());
        }
    });
    let weak = ui.as_weak();
    ui.on_preset_requested(move |preset| {
        if let Some(ui) = weak.upgrade() {
            if !ui.get_connected() || ui.get_fault() {
                return;
            }
            let (temperature, speed, label) = match preset {
                0 => (55.0, 50.0, "Eco"),
                1 => (65.0, 75.0, "Balanced"),
                2 => (78.0, 95.0, "Boost"),
                _ => return,
            };
            ui.set_setpoint(temperature);
            ui.set_pump_speed(speed);
            ui.set_preset(preset);
            ui.set_status(
                format!("{label} preset · {temperature:.0} °C target / {speed:.0} % pump speed")
                    .into(),
            );
        }
    });

    let weak = ui.as_weak();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            let Some(ui) = weak.upgrade() else { return };
            if ui.get_paused() || !ui.get_connected() {
                return;
            }
            let mut state = simulation.borrow_mut();
            state.seconds += 1;
            let time = state.seconds as f32;
            let demand = ui.get_setpoint() + if ui.get_fault() { 28.0 } else { 0.0 };
            state.temperature += (demand - state.temperature) * 0.08 + 0.25 * (time / 8.0).sin();
            let pressure = 62.0
                + (ui.get_setpoint() - 65.0) * 0.45
                + (ui.get_pump_speed() - 75.0) * 0.85
                + 7.0 * (time / 11.0).sin();
            let flow = if ui.get_fault() {
                0.0
            } else {
                120.0 * ui.get_pump_speed() / 75.0 + 12.0 * (time / 9.0).sin()
            };
            ui.set_temperature(state.temperature);
            ui.set_pressure(pressure);
            ui.set_flow(flow);
            ui.set_tank_level(68.0 + 3.0 * (time / 32.0).sin());
            ui.set_cooling_duty(
                (45.0 + (state.temperature - ui.get_setpoint()) * 4.0).clamp(0.0, 100.0),
            );
            let power = if ui.get_fault() {
                0.0
            } else {
                2.4 * (ui.get_pump_speed() / 100.0).powi(3)
            };
            ui.set_energy(ui.get_energy() + power / 3600.0);
            ui.set_elapsed(
                format!(
                    "{:02}:{:02}:{:02}",
                    state.seconds / 3600,
                    state.seconds / 60 % 60,
                    state.seconds % 60
                )
                .into(),
            );
            ui.set_time_minimum(time - 120.0);
            ui.set_time_maximum(time);
            append(
                &temperature,
                TrendSample {
                    time,
                    value: state.temperature,
                },
            );
            append(
                &target,
                TrendSample {
                    time,
                    value: ui.get_setpoint(),
                },
            );
            append(
                &pressure_history,
                TrendSample {
                    time,
                    value: pressure,
                },
            );
            let high = pressure >= 80.0;
            if high && !state.pressure_high {
                let seconds = state.seconds;
                state.alarms.raise(
                    "Pressure above warning limit",
                    "Pressure / PT-02",
                    AlarmSeverity::Warning,
                    seconds,
                );
            } else if !high && state.pressure_high {
                state.alarms.clear_source("Pressure / PT-02");
            }
            if high != state.pressure_high {
                ui.set_alarms(state.alarms.visible(ui.get_alarm_filter()));
                ui.set_unacknowledged(state.alarms.unacknowledged_count());
            }
            state.pressure_high = high;
        },
    );

    ui.run()
}
