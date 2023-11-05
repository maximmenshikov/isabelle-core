extern crate serde_json;
use std::path::Path;

use crate::state::store::Store;
use isabelle_dm::data_model::item::*;
use log::{error, info};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct StoreLocal {
    pub path: String,
    pub collections: HashMap<String, u64>,
    pub items: HashMap<u64, HashMap<u64, bool>>,
    pub items_count: HashMap<u64, u64>,
}

unsafe impl Send for StoreLocal {}

impl StoreLocal {
    pub fn new() -> Self {
        Self {
            path: "".to_string(),
            collections: HashMap::new(),
            items: HashMap::new(),
            items_count: HashMap::new(),
        }
    }
}

impl Store for StoreLocal {
    fn connect(&mut self, url: &str) {
        self.path = url.to_string();
        let collections = fs::read_dir(self.path.to_string() + "/collection").unwrap();
        for coll in collections {
            let idx = coll.as_ref().unwrap().file_name().into_string().unwrap();
            let new_col: HashMap<u64, bool> = HashMap::new();
            let coll_index = self.items.len().try_into().unwrap();
            self.items.insert(coll_index, new_col);
            self.collections.insert(idx.clone(), coll_index);
            info!("New collection {}", idx.clone());

            let cnt_str =
                std::fs::read_to_string(self.path.clone() + "/collection/" + &idx + "/cnt");
            if let Err(_e) = cnt_str {
                error!("Failed to read counter");
                continue;
            }

            let parsed = cnt_str.as_ref().unwrap().trim().parse::<u64>();
            if let Err(_e) = parsed {
                error!("Failed to parse counter {}", cnt_str.as_ref().unwrap());
                continue;
            }

            self.items_count
                .insert(self.collections[&idx], *parsed.as_ref().unwrap());
            info!(" - index: {}", self.collections[&idx]);
            info!(" - counter: {}", parsed.as_ref().unwrap());

            let data_files = fs::read_dir(self.path.to_string() + "/collection/" + &idx).unwrap();
            for data_file in data_files {
                let data_file_idx = data_file.as_ref().unwrap().file_name().into_string().unwrap();
                let tmp_path = self.path.to_string() + "/collection/" + &idx + "/" + &data_file_idx;
                if Path::new(&tmp_path).is_dir() {
                    let m = self.items.get_mut(&coll_index).unwrap();
                    (*m).insert(data_file_idx.parse::<u64>().unwrap(), true);
                    info!("{}: idx {}", &idx, &data_file_idx);
                }
            }
        }
    }

    fn disconnect(&mut self) {}

    fn get_collections(&mut self) -> Vec<String> {
        let mut lst: Vec<String> = Vec::new();

        for coll in &self.collections {
            lst.push(coll.0.clone());
        }

        return lst;
    }

    fn get_item_ids(&mut self, collection: &str) -> HashMap<u64, bool> {
        if !self.collections.contains_key(collection) {
            return HashMap::new();
        }

        let coll_id = self.collections[collection];
        return self.items[&coll_id].clone();
    }

    fn get_all_items(&mut self, collection: &str) -> HashMap<u64, Item> {
        return self.get_items(collection, u64::MAX, u64::MAX, u64::MAX);
    }

    fn get_item(&mut self, collection: &str, id: u64) -> Option<Item> {
        let tmp_path = self.path.to_string() +
            "/collection/" + collection +
            "/" + &id.to_string() + "/data.js";
        if Path::new(&tmp_path).is_file() {
            let text = std::fs::read_to_string(tmp_path).unwrap();
            let itm: Item = serde_json::from_str(&text).unwrap();
            return Some(itm);
        }
        return None;
    }

    fn get_items(
        &mut self,
        collection: &str,
        id_min: u64,
        id_max: u64,
        limit: u64,
    ) -> HashMap<u64, Item> {
        let mut map: HashMap<u64, Item> = HashMap::new();
        let itms = self
            .items
            .get_mut(&self.collections[collection])
            .unwrap()
            .clone();
        let mut eff_id_min = id_min;
        let eff_id_max = id_max;
        let mut count = 0;

        if eff_id_min == u64::MAX {
            eff_id_min = 0;
        }

        info!("Getting {} in range {} - {} limit {}", &collection, eff_id_min, eff_id_max, limit);
        for itm in itms {
            if itm.0 >= eff_id_min && itm.0 <= eff_id_max {
                let new_item = self.get_item(collection, itm.0);
                if !new_item.is_none() {
                    map.insert(itm.0, new_item.unwrap());
                    count = count + 1;
                    if count >= limit {
                        break;
                    }
                }
            }
        }
        info!(" - result: {} items", count);

        return map;
    }

    fn set_item(&mut self, collection: &str, itm: &Item, merge: bool) {
        let old_itm = self.get_item(collection, itm.id);
        let mut new_itm = itm.clone();
        if !old_itm.is_none() && merge {
            new_itm = old_itm.unwrap().clone();
            new_itm.merge(itm);
        }
        let tmp_path =
            self.path.to_string() + "/collection/" + collection + "/" + &new_itm.id.to_string();

        let _dir_create_err = std::fs::create_dir(&tmp_path);

        let tmp_data_path = tmp_path.clone() + "/data.js";
        let s = serde_json::to_string(&new_itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");

        let coll_id = self.collections[collection];
        if self.items.contains_key(&coll_id) {
            let coll = self.items.get_mut(&coll_id).unwrap();
            if coll.contains_key(&new_itm.id) {
                *(coll.get_mut(&new_itm.id).unwrap()) = true;
            } else {
                coll.insert(new_itm.id, true);
            }
            if self.items_count.contains_key(&coll_id) {
                let cnt = self.items_count.get_mut(&coll_id).unwrap();
                if new_itm.id >= *cnt {
                    *cnt = new_itm.id + 1;
                    let _res = std::fs::write(
                        self.path.to_string() + "/collection/" + collection + "/cnt",
                        (new_itm.id + 1).to_string(),
                    );
                }
            } else {
                self.items_count.insert(coll_id, new_itm.id + 1);
                let _res = std::fs::write(
                    self.path.to_string() + "/collection/" + collection + "/cnt",
                    (new_itm.id + 1).to_string(),
                );
            }
        }
    }

    fn del_item(&mut self, collection: &str, id: u64) -> bool {
        let tmp_path = self.path.to_string() + "/" + collection + "/" + &id.to_string();
        let path = Path::new(&tmp_path);
        if path.exists() {
            let _res = std::fs::remove_dir_all(tmp_path);
        }
        let coll_id = self.collections[collection];
        if self.items.contains_key(&coll_id) {
            let coll = self.items.get_mut(&coll_id).unwrap();
            if coll.contains_key(&id) {
                coll.remove(&id);
                return true;
            }
        }
        return false;
    }

    fn get_credentials(&mut self) -> String {
        return self.path.clone() + "/credentials.json";
    }

    fn get_pickle(&mut self) -> String {
        return self.path.clone() + "/token.pickle";
    }

    fn get_internals(&mut self) -> Item {
        let tmp_data_path = self.path.clone() + "/internals.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    fn get_settings(&mut self) -> Item {
        let tmp_data_path = self.path.clone() + "/settings.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    fn set_settings(&mut self, itm: Item) {
        let tmp_data_path = self.path.clone() + "/settings.js";
        let s = serde_json::to_string(&itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");
    }
}
