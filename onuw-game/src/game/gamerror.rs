use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Excepted {} Roles due to having {players} players, but got {roles} Roles instead.", players + 3)]
    InvalidRoleCount { roles: usize, players: usize },
}
