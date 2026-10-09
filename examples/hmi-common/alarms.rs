//! Small host-owned store shared by the examples. The pack stays pure Slint.
use crate::{Alarm, AlarmFilter, AlarmSeverity};
use slint::{ModelRc, VecModel};
use std::rc::Rc;

pub struct AlarmStore {
    records: Vec<Alarm>,
    next_id: u64,
}

impl AlarmStore {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            next_id: 1,
        }
    }

    pub fn raise(&mut self, title: &str, source: &str, severity: AlarmSeverity, seconds: u64) {
        self.records.insert(
            0,
            Alarm {
                id: format!("alarm-{}", self.next_id).into(),
                title: title.into(),
                source: source.into(),
                timestamp: format!(
                    "{:02}:{:02}:{:02}",
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                )
                .into(),
                severity,
                acknowledged: false,
                active: true,
            },
        );
        self.next_id += 1;
        // This demo keeps a bounded history. Production storage belongs to the host.
        self.records.truncate(32);
    }

    pub fn clear_source(&mut self, source: &str) {
        for alarm in &mut self.records {
            if alarm.source == source {
                alarm.active = false;
            }
        }
    }

    pub fn acknowledge(&mut self, id: &str) {
        if let Some(alarm) = self.records.iter_mut().find(|alarm| alarm.id == id) {
            alarm.acknowledged = true;
        }
    }

    pub fn unacknowledged_count(&self) -> i32 {
        self.records
            .iter()
            .filter(|alarm| !alarm.acknowledged)
            .count() as i32
    }

    pub fn visible(&self, filter: AlarmFilter) -> ModelRc<Alarm> {
        Rc::new(VecModel::from(
            self.records
                .iter()
                .filter(|alarm| match filter {
                    AlarmFilter::All => true,
                    AlarmFilter::Unacknowledged => !alarm.acknowledged,
                    AlarmFilter::Warning => alarm.severity == AlarmSeverity::Warning,
                    AlarmFilter::Critical => alarm.severity == AlarmSeverity::Critical,
                })
                .cloned()
                .collect::<Vec<_>>(),
        ))
        .into()
    }
}
