use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Excepted {} Roles due to having {players} players, but got {roles} Roles instead.", players + 3)]
    InvalidRoleCount { roles: usize, players: usize },
    #[error("No more night actions")]
    NoMoreNightActions,
    #[error("Can't caculate dead players without votes!")]
    WrongCmdOrder,
}
