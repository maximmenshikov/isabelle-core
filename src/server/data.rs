use isabelle_dm::data_model::schedule_entry::*;
use isabelle_dm::data_model::item::*;
use isabelle_dm::data_model::all_settings::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Data {
    pub items_cnt: u64,
    pub items: HashMap<u64, Item>,

    pub schedule_entry_cnt: u64,
    pub schedule_entries: HashMap<u64, ScheduleEntry>,
    pub schedule_entry_times: HashMap<u64, Vec<u64>>,

    pub settings: AllSettings,

    pub gc_path: String,
    pub py_path: String,
    pub data_path: String,
    pub public_url: String,
    pub port: u16,
}

impl Data {
    pub fn new() -> Self {
        Self {
            items_cnt: 0,
            items: HashMap::new(),

            schedule_entry_cnt: 0,
            schedule_entries: HashMap::new(),

            schedule_entry_times: HashMap::new(),

            settings: AllSettings::new(),

            gc_path: "".to_string(),
            py_path: "".to_string(),
            data_path: "".to_string(),
            public_url: "".to_string(),
            port: 8090,
        }
    }
}
