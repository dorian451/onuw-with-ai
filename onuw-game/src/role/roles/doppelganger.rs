use super::{common::get_role_from_chosen_target_player, villager::Villager};
use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    playerinterface::roletarget::RoleTarget,
    role::{roletype::RoleType, ActionFn, ActionFnMap, ActionPriority, Role},
};
use async_trait::async_trait;
use futures::Future;
use once_cell::sync::Lazy;
use std::{fmt::Display, pin::Pin, vec::Vec};
use tracing::{instrument, warn};

static ACTIONS: Lazy<ActionFnMap<Doppelganger>> = Lazy::new(|| {
    [(
        "1".to_string(),
        Doppelganger::night_action as ActionFn<Doppelganger>,
    )]
    .into_iter()
    .collect()
});

#[derive(Clone, Debug)]
pub struct Doppelganger {
    copied: Option<Box<dyn Role>>,
}

#[async_trait]
impl Role for Doppelganger {
    #[instrument(level = "trace")]
    fn new() -> Self {
        Self { copied: None }
    }

    #[instrument(level = "trace")]
    fn id(&self) -> String {
        "Doppelganger".to_string()
    }

    #[instrument(level = "trace")]
    fn verbose_id(&self) -> String {
        format!(
            "{}({})",
            self.id(),
            self.copied
                .as_ref()
                .map(|r| r.verbose_id())
                .unwrap_or_default()
        )
    }

    #[instrument(level = "trace")]
    fn effective_id(&self) -> String {
        if self.copied.is_some() {
            self.copied.as_ref().unwrap().id()
        } else {
            self.id()
        }
    }

    #[instrument(level = "trace")]
    fn role_type(&self) -> RoleType {
        if let Some(role) = &self.copied {
            role.role_type()
        } else {
            RoleType::Villager
        }
    }

    #[instrument(level = "trace", skip(dead))]
    fn win_condition(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        dead: &[(GamePlayer, GameRole)],
    ) -> bool {
        if let Some(role) = &self.copied {
            role.win_condition(game, player, dead)
        } else {
            Villager::eval_wincon(game, dead)
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
        } else if self.copied.is_some() {
            self.copied
                .as_mut()
                .unwrap()
                .action_at_priority(priority, game, player)
                .await
        } else {
            Err(format!("No action for {:?} at priority {}", self, priority))
        }
    }
}

impl Doppelganger {
    #[instrument(level = "trace")]
    fn night_action<'a>(
        &'a mut self,
        game: &'a mut ONUWGame,
        player: &'a GamePlayer,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>> {
        Box::pin(async {
            let other_players = game.all_other_players(player);
            // look at player
            let (target, target_role) =
                get_role_from_chosen_target_player(game, player, &other_players).await;

            let target_role = target_role.clone();

            self.copied = Some(target_role.read().await.clone());
            let copied_role = self.copied.as_mut().unwrap().as_mut();

            game.update_player_type(player, &RoleType::Villager, copied_role.role_type());

            player
                .show_role(
                    RoleTarget::Player(target),
                    target_role.read().await.as_ref(),
                )
                .await;
            player
                .show_role(RoleTarget::Player(player.clone()), self)
                .await;

            let copied_role = self.copied.as_mut().unwrap().as_mut(); //to allow immutable borrow above

            let pri = copied_role
                .priorities()
                .first()
                .unwrap()
                .to_owned()
                .to_owned();

            match copied_role.id().as_str() {
                "Minion" | "Seer" | "Robber" | "Troublemaker" | "Drunk" => {
                    copied_role
                        .action_at_priority(&pri, game, player)
                        .await
                        .unwrap();
                }
                "Insomniac" | "Mason" | "Werewolf" => {
                    let game_role = game.players().get(player).unwrap().clone();
                    game.add_night_action_at_priority(&pri, &game_role, Some(player))
                }
                _ => (),
            };
        })
    }
}
