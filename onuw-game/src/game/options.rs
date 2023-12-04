use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Getters)]
pub struct Options {
    lone_wolf: bool,
    debug_set_roles: bool,
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_lone_wolf(mut self) -> Self {
        self.lone_wolf = true;
        self
    }

    #[allow(unused)]
    pub(crate) fn debug_with_set_roles(mut self) -> Self {
        self.debug_set_roles = true;
        self
    }
}
