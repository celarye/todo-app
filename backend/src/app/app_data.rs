use std::sync::Arc;

use crate::Logic;

pub struct AppData {
    pub logic: Arc<Logic>,
}

impl AppData {
    pub fn new(logic: Arc<Logic>) -> Self {
        AppData { logic }
    }
}
