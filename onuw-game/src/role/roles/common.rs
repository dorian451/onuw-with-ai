use std::collections::HashSet;

use crate::{
    game::{GamePlayer, GameRole, ONUWGame},
    playerinterface::roletarget::RoleTarget,
    role::{roletype::RoleType, Role},
};
use futures::{
    stream::{self},
    StreamExt,
};
use tracing::instrument;

#[instrument(level = "trace")]
pub async fn show_type(
    calling_role: &dyn Role,
    game: &ONUWGame,
    player: &GamePlayer,
    role_type: &RoleType,
) -> usize {
    let mut count = 0;

    stream::iter(
        game.players_by_type()
            .get(role_type)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter(|v| v.as_ref() != player.as_ref()),
    )
    .inspect(|_| count += 1)
    .for_each_concurrent(None, |v| async {
        player
            .show_role_type(RoleTarget::Player(v.clone()), role_type)
            .await
            .unwrap();
    })
    .await;

    count
}

#[instrument(level = "trace", skip(role))]
pub async fn show_role(game: &ONUWGame, player: &GamePlayer, role: &dyn Role) {
    stream::iter(game.players())
        .for_each(|(p, r)| async {
            if let Ok(r) = r.try_read() {
                if r.as_ref() == role && p.as_ref() != player.as_ref() {
                    player
                        .show_role(RoleTarget::Player(p.clone()), role)
                        .await
                        .unwrap();
                }
            }
        })
        .await;
}

#[instrument(level = "trace")]
pub async fn swap_role_with_target_player_with_asker<'a>(
    calling_role: &dyn Role,
    game: &'a mut ONUWGame,
    player: &'a GamePlayer,
    asker: &'a GamePlayer,
    choices: &'a [&'a GamePlayer],
) -> (GameRole, GameRole) {
    let (target, targetrole);
    let orig_role;

    {
        let (one, two) = get_role_from_chosen_target_player(game, asker, choices).await;
        (target, targetrole) = (one, two.clone());
        orig_role = game.players().get(player).unwrap().clone();
    }

    game.change_role(&RoleTarget::Player(player.clone()), &targetrole)
        .await;

    game.update_player_type(
        player,
        &calling_role.role_type(),
        targetrole.try_read().unwrap().role_type(),
    );

    game.change_role(&RoleTarget::Player(target.clone()), &orig_role)
        .await;

    game.update_player_type(
        &target,
        &targetrole.try_read().unwrap().role_type(),
        calling_role.role_type(),
    );

    (targetrole, orig_role)
}

#[instrument(level = "trace")]
pub async fn swap_role_with_target_player<'a>(
    calling_role: &dyn Role,
    game: &'a mut ONUWGame,
    player: &'a GamePlayer,
    choices: &'a [&'a GamePlayer],
) -> (GameRole, GameRole) {
    return swap_role_with_target_player_with_asker(calling_role, game, player, player, choices)
        .await;
}

#[instrument(level = "trace")]
pub async fn get_role_from_chosen_target_player<'a>(
    game: &'a ONUWGame,
    player: &GamePlayer,
    choices: &'a [&'a GamePlayer],
) -> (GamePlayer, &'a GameRole) {
    let target = player.choose_player(choices).await.unwrap();

    let targetrole = game
        .players()
        .get(&target)
        .unwrap_or_else(|| panic!("Why does player {} not have a role?", target));

    (target, targetrole)
}
