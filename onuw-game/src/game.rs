mod gamerror;
mod nightaction;

pub mod options;
pub mod time;
pub(crate) mod voteaction;

use self::nightaction::NightAction;
use self::options::Options;
use self::time::ONUWTime;
use crate::game::voteaction::ONUWGameVoteAction;
use crate::playerinterface::message::Message;
use crate::playerinterface::roletarget::RoleTarget;
use crate::playerinterface::{error, PlayerInterface};
use crate::role::{roletype::RoleType, ActionPriority, Role};
use derive_getters::Getters;
use futures::future::join_all;
use futures::stream;
use futures::Future;
use futures::StreamExt;
use gamerror::GameError;
use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashSet;
use std::fmt::Debug;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::debug;
use tracing::{error, instrument, warn};

pub type GamePlayer = Arc<dyn PlayerInterface>;
pub type GameRole = Arc<RwLock<Box<dyn Role>>>;

#[derive(Getters)]
pub struct ONUWGame {
    players: HashMap<GamePlayer, GameRole>,
    // roles: MultiSet<GameRole>,
    players_by_type: HashMap<RoleType, HashSet<GamePlayer>>,
    centerroles: Vec<GameRole>,
    options: Options,
    nightactions: BTreeMap<ActionPriority, Vec<NightAction>>,
    votes: Option<HashMap<Arc<dyn PlayerInterface>, Arc<dyn PlayerInterface>>>,
    dead: Option<HashSet<Arc<dyn PlayerInterface>>>,
    winners: Option<HashSet<Arc<dyn PlayerInterface>>>,
}

impl ONUWGame {
    #[instrument(level = "trace")]
    pub async fn new(
        players: Vec<GamePlayer>,
        roles: Vec<Box<dyn Role>>,
        options: Options,
    ) -> Result<Self, GameError> {
        if players.len() != roles.len() - 3 {
            Err(GameError::InvalidRoleCount {
                roles: roles.len(),
                players: players.len(),
            })
        } else {
            let mut shuffled_roles: Vec<_> =
                roles.into_iter().map(RwLock::new).map(Arc::new).collect();

            if !options.debug_set_roles() {
                shuffled_roles.as_mut_slice().shuffle(&mut thread_rng());
            }

            let mut rolesitr = shuffled_roles.into_iter();
            let mut assigned_roles = HashMap::new();

            players.into_iter().for_each(|player| {
                assigned_roles.insert(player, rolesitr.next().unwrap());
            });

            let remaining_roles: Vec<_> = rolesitr.collect();

            let mut game = Self {
                // roles: roles.into_iter().collect(),
                players_by_type: stream::iter(assigned_roles.clone())
                    .fold(
                        HashMap::<_, HashSet<_>>::new(),
                        |mut map: _, (player, role)| async move {
                            map.entry(role.read().await.role_type())
                                .and_modify(|v: &mut _| {
                                    v.insert(player.clone());
                                })
                                .or_insert([player.clone()].into_iter().collect());
                            map
                        },
                    )
                    .await,
                centerroles: remaining_roles.clone(),
                options,
                nightactions: BTreeMap::new(),
                players: assigned_roles.clone(),
                votes: None,
                dead: None,
                winners: None,
            };

            let mut assigned_roles_iter = stream::iter(assigned_roles);

            while let Some((player, role)) = assigned_roles_iter.next().await {
                game.add_night_action(&role, Some(&player)).await;
            }

            for role in remaining_roles {
                game.add_night_action(&role, None).await;
            }

            Ok(game)
        }
    }

    #[instrument(level = "trace")]
    pub(crate) async fn add_night_action(&mut self, role: &GameRole, player: Option<&GamePlayer>) {
        for priority in role.read().await.priorities() {
            self.add_night_action_at_priority(priority, role, player);
        }
    }

    #[instrument(level = "trace")]
    pub(crate) fn add_night_action_at_priority(
        &mut self,
        priority: &ActionPriority,
        role: &GameRole,
        player: Option<&GamePlayer>,
    ) {
        let val = match player {
            Some(p) => NightAction::Real(role.clone(), p.clone()),
            None => NightAction::Fake(role.clone()),
        };

        self.nightactions
            .entry(priority.clone())
            .and_modify(|e| e.push(val.clone()))
            .or_insert_with(|| vec![val]);
    }

    #[instrument(level = "trace" skip(onfake))]
    pub async fn perform_next_night_action<Fut: Future<Output = ()>>(
        &mut self,
        onfake: fn() -> Fut,
    ) -> Result<(), GameError> {
        let (priority, actions) = self
            .nightactions
            .pop_first()
            .ok_or(GameError::NoMoreNightActions)?;

        let mut last_role = None;

        for action in actions {
            match action {
                NightAction::Real(role, player) => {
                    let mut role = role.write().await;

                    if last_role != Some(role.effective_id()) {
                        last_role = Some(role.effective_id());
                        self.announce_time(&ONUWTime::Night(role.as_ref())).await;
                    }

                    debug!("performing action of {:?} for player {:?}", role, player);
                    role.action_at_priority(&priority, self, &player).await;
                }
                NightAction::Fake(role) => {
                    let role = role.read().await;

                    if last_role != Some(role.effective_id()) {
                        last_role = Some(role.effective_id());
                        self.announce_time(&ONUWTime::Night(role.as_ref())).await;
                    }

                    debug!("skipping action of {:?}", role);
                    onfake().await;
                }
            }
        }

        Ok(())
    }

    #[instrument(level = "trace")]
    pub fn peek_next_night_action(&self) -> Option<&ActionPriority> {
        self.nightactions.first_key_value().map(|(k, _)| k)
    }

    #[instrument(level = "trace")]
    pub(crate) fn all_other_players(&self, player: &GamePlayer) -> Vec<&GamePlayer> {
        self.players
            .iter()
            .filter_map(|(p, _)| {
                if p.as_ref() == player.as_ref() {
                    None
                } else {
                    Some(p)
                }
            })
            .collect()
    }

    #[instrument(level = "trace")]
    pub async fn show_all_roles(&self) {
        let fut = join_all(self.players.iter().map(|(player, role)| async {
            player
                .show_role(
                    RoleTarget::Player(player.clone()),
                    role.read().await.as_ref(),
                )
                .await;
        }));
        fut.await;
    }

    #[instrument(level = "trace")]
    pub(crate) async fn change_role(&mut self, target: &RoleTarget, role: &GameRole) {
        match target {
            RoleTarget::Player(player) => {
                self.players
                    .insert(player.clone(), role.clone())
                    .unwrap_or_else(|| panic!("Why did {} not have a role?", player));
            }

            RoleTarget::Center(i) => {
                self.centerroles[*i] = role.clone();
            }
        }
    }

    #[instrument(level = "trace")]
    pub(crate) fn update_player_type(
        &mut self,
        target: &GamePlayer,
        old_type: &RoleType,
        new_type: RoleType,
    ) {
        self.players_by_type
            .get_mut(old_type)
            .unwrap()
            .remove(target);

        self.players_by_type
            .entry(new_type)
            .and_modify(|e| {
                e.insert(target.clone());
            })
            .or_insert_with(|| HashSet::from_iter([target.clone()].into_iter()));
    }

    #[instrument(level = "trace")]
    pub async fn send_handshake(&self) {
        let roles = stream::iter(self.centerroles.iter())
            .then(|i| async move { i.read().await.id() })
            .chain(stream::iter(self.players.values()).then(|r| async { r.read().await.id() }))
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .counts();

        join_all(
            self.players()
                .iter()
                .map(|(p, _)| async { p.handshake(&self.all_other_players(p), &roles).await }),
        )
        .await;
    }

    #[instrument(level = "trace")]
    pub async fn send_message_to_players(&self, message: Message) {
        join_all(
            self.players()
                .iter()
                .filter(|(p, _)| p.as_ref() != message.sender.as_ref())
                .map(|(p, _)| p.receive_message(&message)),
        )
        .await;
    }

    #[instrument(level = "trace")]
    pub async fn collect_votes(&mut self) -> Result<(), GameError> {
        if self.votes.is_some() || self.dead.is_some() || self.winners.is_some() {
            (Err(GameError::WrongCmdOrder))?
        }

        let immut_self: &_ = self;
        let v = Some(
            join_all(immut_self.players.keys().map(|v| async move {
                (
                    v.clone(),
                    v.choose_player(&immut_self.all_other_players(v))
                        .await
                        .unwrap_or_else(|e| {
                            error!("{:#?}", e);
                            todo!()
                        }),
                )
            }))
            .await
            .into_iter()
            .collect(),
        );

        self.votes = v;

        Ok(())
    }

    #[instrument(level = "trace")]
    pub async fn calc_dead_and_winners(&mut self) -> Result<(), GameError> {
        if self.votes.is_none() || self.dead.is_some() || self.winners.is_some() {
            (Err(GameError::WrongCmdOrder))?
        }

        self.dead = Some(
            self.votes
                .as_ref()
                .unwrap()
                .values()
                .cloned()
                .counts()
                .into_iter()
                .map(|(k, v)| (v, k))
                .max_set_by(|(a, _), (b, _)| a.cmp(b))
                .into_iter()
                .map(|(_, p)| p)
                .collect(),
        );

        debug!("dead before actions:\n{:#?}", self.dead.as_ref().unwrap());

        let dead_vec = self
            .dead
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let actions: Vec<_> = stream::iter(self.players.iter())
            .then(|(p, r)| async {
                stream::iter(r.read().await.after_vote(
                    self,
                    p,
                    self.votes.as_ref().unwrap(),
                    &dead_vec,
                ))
            })
            .flatten()
            .collect()
            .await;

        for action in actions {
            match action {
                ONUWGameVoteAction::Kill(p) => {
                    self.dead.as_mut().unwrap().insert(p);
                }
            }
        }

        debug!("dead:\n{:#?}", self.dead.as_ref().unwrap());

        let dead_with_roles: Vec<_> = self
            .dead
            .as_ref()
            .unwrap()
            .iter()
            .map(|d| (d.clone(), self.players.get(d).unwrap().clone()))
            .collect();

        self.winners = Some(
            stream::iter(self.players.iter())
                .filter_map(|(player, role)| async {
                    let win = role
                        .read()
                        .await
                        .win_condition(self, player, &dead_with_roles);

                    if win {
                        Some(player.clone())
                    } else {
                        None
                    }
                })
                .collect()
                .await,
        );

        stream::iter(self.players.iter())
            .for_each(|(player, role)| async {
                player
                    .show_win(
                        role.read()
                            .await
                            .win_condition(self, player, &dead_with_roles),
                    )
                    .await;
            })
            .await;

        Ok(())
    }

    #[instrument(level = "trace")]
    async fn announce_time<'a>(&self, time: &'a ONUWTime<'a>) {
        join_all(self.players().iter().map(|(p, _)| p.show_time(time))).await;
    }
}

impl Debug for ONUWGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ONUWGame").finish()
    }
}
