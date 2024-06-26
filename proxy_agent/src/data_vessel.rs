// Copyright (c) Microsoft Corporation
// SPDX-License-Identifier: MIT

use crate::{common::logger, key_keeper::key::Key};
use std::sync::mpsc::Sender;

pub enum DataAction {
    Stop,
    KeyKeeperSetKey {
        key: Key,
        response: std::sync::mpsc::Sender<bool>,
    },
    KeyKeeperGetKeyValue {
        response: std::sync::mpsc::Sender<Key>,
    },
}

#[derive(Clone, Debug)]
pub struct DataVessel {
    sender: Sender<DataAction>,
}
impl DataVessel {
    pub fn start_new_async() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<DataAction>();

        std::thread::spawn(move || {
            // chached data are defined here
            let mut cached_key: Key = Key::empty(); // start with empyt key

            while let Ok(action) = receiver.recv() {
                match action {
                    DataAction::Stop => {
                        break;
                    }
                    DataAction::KeyKeeperSetKey { key, response } => {
                        // Set the key from the key keeper
                        cached_key = key.clone();
                        _ = response.send(true);
                    }
                    DataAction::KeyKeeperGetKeyValue { response } => {
                        // Get the key from the key keeper
                        _ = response.send(cached_key.clone());
                    }
                }
            }
        });

        DataVessel { sender }
    }

    pub fn stop(&self) {
        let _ = self.sender.send(DataAction::Stop);
    }
}

/// KeyKeeper implementation
impl DataVessel {
    pub fn update_current_key(&self, key: Key) -> bool {
        let (response, receiver) = std::sync::mpsc::channel::<bool>();
        let _ = self
            .sender
            .send(DataAction::KeyKeeperSetKey { key, response });
        match receiver.recv() {
            Ok(result) => result,
            Err(e) => {
                logger::write_warning(format!("Failed to update current key with error: {e}"));
                false
            }
        }
    }

    fn get_current_key(&self) -> Key {
        let (response, receiver) = std::sync::mpsc::channel::<Key>();
        let _ = self
            .sender
            .clone()
            .send(DataAction::KeyKeeperGetKeyValue { response });
        match receiver.recv() {
            Ok(key) => key,
            Err(e) => {
                logger::write_warning(format!("Failed to get current key with error: {e}"));
                // return empty key if failed to get the key
                Key::empty()
            }
        }
    }

    pub fn get_current_key_value(&self) -> String {
        self.get_current_key().key
    }

    pub fn get_current_key_guid(&self) -> String {
        self.get_current_key().guid
    }

    pub fn get_current_key_incarnation(&self) -> Option<u32> {
        self.get_current_key().incarnationId
    }
}
