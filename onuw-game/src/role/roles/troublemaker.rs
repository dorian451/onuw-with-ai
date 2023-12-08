use super::{common::swap_role_with_target_player_with_asker, villager::Villager};
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use once_cell::sync::Lazy;
use std::pin::Pin;
use tracing::instrument;

static ACTIONS: Lazy<ActionFnMap<Troublemaker>> = Lazy::new(|| [(
        "7".to_string(),
        Troublemaker::night_action as ActionFn<Troublemaker>
    )].into_iter().collect());

#[derive(Clone, Debug)]
pub struct Troublemaker;

#[async_trait]
impl Role for Troublemaker {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Troublemaker".to_string()
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

impl Troublemaker {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            if player.choose_bool().await.unwrap() {
                let playerlist: Vec<_> = game
                    .all_other_players(player)
                    .into_iter()
                    .cloned()
                    .collect();

                let target_1 = player
                    .choose_player(playerlist.iter().collect::<Vec<_>>().as_slice())
                    .await
                    .unwrap();
                let target_1_role = game.players().get(&target_1).unwrap().clone();

                swap_role_with_target_player_with_asker(
                    target_1_role.try_read().unwrap().as_ref(),
                    game,
                    &target_1,
                    player,
                    playerlist
                        .iter()
                        .filter(|v| v.as_ref() != target_1.as_ref())
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .await;
            }
        })
    }
}



