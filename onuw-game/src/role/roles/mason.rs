use super::{common::show_role, villager::Villager};
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use once_cell::sync::Lazy;
use std::pin::Pin;
use tracing::instrument;

static ACTIONS: Lazy<ActionFnMap<Mason>> = Lazy::new(|| [("4".to_string(), Mason::night_action as ActionFn<Mason>)].into_iter().collect());

#[derive(Clone, Debug)]
pub struct Mason;

#[async_trait]
impl Role for Mason {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn min_amt() -> usize where Self:Sized {
        2
    }

    #[instrument(level = "trace")]
    fn max_amt() -> usize where Self:Sized {
        2
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Mason".to_string()
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

impl Mason {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            show_role(game, player, self).await;
        })
    }
}



