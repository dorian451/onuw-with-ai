use super::common::show_type;
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
    static ref ACTIONS: ActionFnMap<Werewolf> = [(
        "2".to_string(),
        Werewolf::night_action as ActionFn<Werewolf>
    )]
    .into_iter()
    .collect();
}

#[derive(Clone, Debug)]
pub struct Werewolf;

#[async_trait]
impl Role for Werewolf {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Werewolf".to_string()
    }

    #[instrument(level = "trace")]
    fn role_type(&self) -> RoleType {
        RoleType::Werewolf
    }

    #[instrument(level = "trace", skip(dead))]
    fn win_condition(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        dead: &[(GamePlayer, GameRole)],
    ) -> bool {
        dead.iter()
            .all(|(_, r)| r.try_read().unwrap().role_type() != RoleType::Werewolf)
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

impl Werewolf {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            let alone = show_type(self, game, player, &RoleType::Werewolf).await == 0;

            if *game.options().lone_wolf() && alone {
                let i: usize = (player.choose_num(&[0, 1, 2]).await.unwrap())
                    .try_into()
                    .unwrap();
                player
                    .show_role(
                        RoleTarget::Center(i),
                        game.centerroles().get(i).unwrap().read().await.as_ref(),
                    )
                    .await;
            };
        })
    }
}
