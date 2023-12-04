use super::{common::get_role_from_chosen_target_player, villager::Villager};
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    playerinterface::roletarget::RoleTarget,
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use lazy_static::lazy_static;
use std::{pin::Pin, vec::Vec};
use tracing::instrument;

lazy_static! {
    static ref ACTIONS: ActionFnMap<Seer> =
        [("5".to_string(), Seer::night_action as ActionFn<Seer>)]
            .into_iter()
            .collect();
}

#[derive(Clone, Debug)]
pub struct Seer;

#[async_trait]
impl Role for Seer {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Seer".to_string()
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

impl Seer {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            if player.choose_bool().await.unwrap() {
                let other_players = game.all_other_players(player);
                // look at player
                let (target, target_role) =
                    get_role_from_chosen_target_player(game, player, &other_players).await;

                player
                    .show_role(
                        RoleTarget::Player(target),
                        target_role.read().await.as_ref(),
                    )
                    .await;
            } else {
                // look at two center
                let one: usize = player
                    .choose_num(&[0, 1, 2])
                    .await
                    .unwrap()
                    .try_into()
                    .unwrap();

                let two: usize = player
                    .choose_num(
                        (0..=2)
                            .filter(|v| v != &one)
                            .map(|v| v.try_into().unwrap())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .await
                    .unwrap()
                    .try_into()
                    .unwrap();

                player
                    .show_role(
                        RoleTarget::Center(one),
                        game.centerroles()[one].read().await.as_ref(),
                    )
                    .await;

                player
                    .show_role(
                        RoleTarget::Center(two),
                        game.centerroles()[two].read().await.as_ref(),
                    )
                    .await;
            }
        })
    }
}
