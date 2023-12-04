use std::collections::HashMap;

use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    playerinterface::PlayerInterface,
    role::{roletype::RoleType, Role},
};
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Tanner;

impl Role for Tanner {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Tanner".to_string()
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
        Self::eval_wincon(game, player, dead)
    }

    #[instrument(level = "trace", skip(winners))]
    fn prevent_win<'a>(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        winners: &'a [(&'a dyn PlayerInterface, &'a dyn Role)],
        votes: &'a HashMap<GamePlayer, GamePlayer>,
    ) -> Vec<&'a dyn PlayerInterface> {
        if winners.iter().any(|(v, _)| v.name() == player.name()) {
            winners
                .iter()
                .filter(|(_, r)| {
                    r.role_type() == RoleType::Werewolf || r.effective_id() == "Minion"
                })
                .map(|(p, _)| p.to_owned())
                .collect()
        } else {
            Role::prevent_win(self, game, player, winners, votes)
        }
    }
}

impl Tanner {
    #[instrument(level = "trace", skip(dead))]
    pub fn eval_wincon(
        game: &ONUWGame,
        player: &GamePlayer,
        dead: &[(GamePlayer, GameRole)],
    ) -> bool {
        dead.iter().any(|(v, _)| v.name() == player.name())
    }
}
