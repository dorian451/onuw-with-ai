use super::PlayerInterface;
use std::sync::Arc;

#[derive(Debug)]
pub enum RoleTarget {
    Player(Arc<dyn PlayerInterface>),
    Center(usize),
}
