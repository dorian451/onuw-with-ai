use super::villager::Villager;
use crate::{
    game::{voteaction::ONUWGameVoteAction, GamePlayer, GameRole, ONUWGame},
    role::{roletype::RoleType, Role},
};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Hunter;

impl Role for Hunter {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Hunter".to_string()
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
        Villager::eval_wincon(game, dead)
    }

    #[instrument(level = "trace")]
    fn after_vote(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        votes: &HashMap<GamePlayer, GamePlayer>,
        dead: &[GamePlayer],
    ) -> Vec<ONUWGameVoteAction> {
        if dead.iter().any(|v| v.name() == player.name()) {
            vec![ONUWGameVoteAction::Kill(votes.get(player).unwrap().clone())]
        } else {
            Vec::new()
        }
    }
}
