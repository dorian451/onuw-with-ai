use tracing::instrument;

use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    role::{roletype::RoleType, Role},
};

#[derive(Clone, Debug)]
pub struct Villager;

impl Role for Villager {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Villager".to_string()
    }

    #[instrument(level = "trace")]
    fn role_type(&self) -> RoleType {
        RoleType::Villager
    }

    #[instrument(level = "trace", skip(dead))]
    fn win_condition(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        dead: &[(GamePlayer, GameRole)],
    ) -> bool {
        Self::eval_wincon(game, dead)
    }
}

impl Villager {
    #[instrument(level = "trace", skip(dead))]
    pub fn eval_wincon(game: &ONUWGame, dead: &[(GamePlayer, GameRole)]) -> bool {
        (game
            .players_by_type()
            .get(&RoleType::Werewolf)
            .map(|entry| entry.is_empty())
            .unwrap_or(true)
            && dead.is_empty())
            || (dead
                .iter()
                .any(|(_, r)| r.try_read().unwrap().role_type() == RoleType::Werewolf))
    }
}
