use super::villager::Villager;
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    playerinterface::roletarget::RoleTarget,
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use once_cell::sync::Lazy;
use std::pin::Pin;
use tracing::instrument;

static ACTIONS: Lazy<ActionFnMap<Drunk>> = Lazy::new(|| [("8".to_string(), Drunk::night_action as ActionFn<Drunk>)].into_iter().collect());

#[derive(Clone, Debug)]
pub struct Drunk;

#[async_trait]
impl Role for Drunk {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self {}
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Drunk".to_string()
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

impl Drunk {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            let choice: usize = player
                .choose_num(&[0, 1, 2])
                .await
                .unwrap()
                .try_into()
                .unwrap();

            let orig_role = game.players().get(player).unwrap().clone();
            let targetrole = game.centerroles().get(choice).unwrap().clone();

            game.change_role(&RoleTarget::Player(player.clone()), &targetrole)
                .await;

            game.update_player_type(
                player,
                &self.role_type(),
                targetrole.try_read().unwrap().role_type(),
            );

            game.change_role(&RoleTarget::Center(choice), &orig_role)
                .await;
        })
    }
}



