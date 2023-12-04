use super::common::show_type;
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use lazy_static::lazy_static;
use std::pin::Pin;
use tracing::instrument;

lazy_static! {
    static ref ACTIONS: ActionFnMap<Minion> =
        [("3".to_string(), Minion::night_action as ActionFn<Minion>)]
            .into_iter()
            .collect();
}

#[derive(Clone, Debug)]
pub struct Minion;

#[async_trait]
impl Role for Minion {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Minion".to_string()
    }

    #[instrument(level = "trace")]
    fn role_type(&self) -> RoleType {
        RoleType::Other("Minion")
    }

    #[instrument(level = "trace", skip(dead))]
    fn win_condition(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        dead: &[(GamePlayer, GameRole)],
    ) -> bool {
        if game
            .players()
            .iter()
            .any(|(_, r)| r.try_read().unwrap().role_type() == RoleType::Werewolf)
        {
            dead.iter()
                .all(|(_, r)| r.try_read().unwrap().role_type() != RoleType::Werewolf)
        } else {
            dead.iter().all(|(p, _)| p.name() != player.name())
        }
    }

    #[instrument(level = "trace")]
    fn priorities(&self) -> Vec<&ActionPriority> {
        ACTIONS.keys().collect()
    }

    #[instrument(level = "trace")]
    async fn action_at_priority(
        &mut self,
        priority: &ActionPriority,
        game: &mut ONUWGame,
        player: &GamePlayer,
    ) -> Result<(), String> {
        if let Some(action) = ACTIONS.get(priority) {
            action(self, game, player).await;
            Ok(())
        } else {
            Err(format!("No action for {:?} at priority {}", self, priority))
        }
    }
}

impl Minion {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async move {
            show_type(self, game, player, &RoleType::Werewolf).await;
        })
    }
}
