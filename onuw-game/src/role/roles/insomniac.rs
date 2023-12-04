use super::villager::Villager;
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
    static ref ACTIONS: ActionFnMap<Insomniac> = [(
        "9".to_string(),
        Insomniac::night_action as ActionFn<Insomniac>
    )]
    .into_iter()
    .collect();
}

#[derive(Clone, Debug)]
pub struct Insomniac;

#[async_trait]
impl Role for Insomniac {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Insomniac".to_string()
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

impl Insomniac {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            match game.players().get(player).unwrap().try_read() {
                Ok(role) => {
                    player
                        .show_role(RoleTarget::Player(player.clone()), role.as_ref())
                        .await
                }
                Err(_) => {
                    player
                        .show_role(RoleTarget::Player(player.clone()), self)
                        .await
                }
            };
        })
    }
}
