pub mod error;
pub mod message;
pub mod roletarget;

use self::{error::PlayerInterfaceError, message::Message, roletarget::RoleTarget};
use crate::{
    game::time::ONUWTime,
    role::{roletype::RoleType, Role},
};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    sync::Arc,
};

#[async_trait]
pub trait PlayerInterface: Send + Sync {
    fn name(&self) -> &str;

    async fn show_role(
        &self,
        target: RoleTarget,
        role: &dyn Role,
    ) -> Result<(), PlayerInterfaceError>;

    async fn show_role_type(
        &self,
        target: RoleTarget,
        role: &RoleType,
    ) -> Result<(), PlayerInterfaceError>;

    async fn choose_player<'a>(
        &self,
        players: &'a [&'a Arc<dyn PlayerInterface>],
    ) -> Result<Arc<dyn PlayerInterface>, PlayerInterfaceError>;

    async fn choose_bool(&self) -> Result<bool, PlayerInterfaceError>;

    async fn choose_num(&self, choices: &[isize]) -> Result<isize, PlayerInterfaceError>;

    async fn receive_message(&self, message: &Message) -> Result<(), PlayerInterfaceError>;

    async fn handshake<'a>(
        &self,
        players: &'a [&'a Arc<dyn PlayerInterface>],
        roles: &'a HashMap<String, usize>,
    ) -> Result<(), PlayerInterfaceError>;

    async fn show_time(&self, time: &ONUWTime) -> Result<(), PlayerInterfaceError>;

    async fn show_win(&self, won_game: bool) -> Result<(), PlayerInterfaceError>;
}

impl Display for dyn PlayerInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player: {}", self.name())
    }
}

impl Debug for dyn PlayerInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self as &dyn Display).fmt(f)
    }
}

impl PartialEq for dyn PlayerInterface {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for dyn PlayerInterface {}

impl Hash for dyn PlayerInterface {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state)
    }
}

impl PartialOrd for dyn PlayerInterface {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name().cmp(other.name()))
    }
}

impl Ord for dyn PlayerInterface {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
