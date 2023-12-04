use super::{common::swap_role_with_target_player, villager::Villager};
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    playerinterface::roletarget::RoleTarget,
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use lazy_static::lazy_static;
use std::pin::Pin;
use tracing::instrument;

lazy_static! {
    static ref ACTIONS: ActionFnMap<Robber> =
        [("6".to_string(), Robber::night_action as ActionFn<Robber>)]
            .into_iter()
            .collect();
}

#[derive(Clone, Debug)]
pub struct Robber;

#[async_trait]
impl Role for Robber {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Robber".to_string()
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

impl Robber {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            if player.choose_bool().await.unwrap() {
                let choices = game
                    .all_other_players(player)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>();

                let (new_role_player, _) = swap_role_with_target_player(
                    self,
                    game,
                    player,
                    choices.iter().collect::<Vec<_>>().as_slice(),
                )
                .await;
                player
                    .show_role(
                        RoleTarget::Player(player.clone()),
                        new_role_player.read().await.as_ref(),
                    )
                    .await;
            }
        })
    }
}
