pub mod roles;
pub mod roletype;

use self::roletype::RoleType;
use crate::{
    game::{voteaction::ONUWGameVoteAction, GamePlayer, GameRole, ONUWGame},
    playerinterface::PlayerInterface,
};
use async_trait::async_trait;
use dyn_clone::DynClone;
use futures::Future;
use std::{
    boxed::Box,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    pin::Pin,
};
use tracing::instrument;

pub type ActionPriority = String;

pub type ActionFn<R> = for<'a> fn(
    &'a mut R,
    &'a mut ONUWGame,
    &'a GamePlayer,
) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>>;

pub type ActionFnG = Box<
    dyn for<'a> Fn(
            &'a mut dyn Role,
            &'a mut ONUWGame,
            &'a GamePlayer,
        ) -> Pin<Box<dyn Future<Output = ()> + 'a + Send>>
        + Send
        + Sync,
>;

pub type ActionFnMap<R> = HashMap<ActionPriority, ActionFn<R>>;
pub type ActionFnMapG = HashMap<ActionPriority, ActionFnG>;

#[async_trait]
pub trait Role: Send + Sync + Debug + DynClone {
    fn new() -> Self
    where
        Self: Sized;

    fn id(&self) -> String;

    #[instrument(level = "trace")]
    fn verbose_id(&self) -> String {
        self.id()
    }

    #[instrument(level = "trace")]
    fn effective_id(&self) -> String {
        self.id()
    }

    #[instrument(level = "trace")]
    fn min_amt() -> usize
    where
        Self: Sized,
    {
        1
    }

    #[instrument(level = "trace")]
    fn max_amt() -> usize
    where
        Self: Sized,
    {
        1
    }

    fn role_type(&self) -> RoleType;

    #[instrument(level = "trace")]
    fn priorities(&self) -> Vec<&ActionPriority> {
        Vec::new()
    }

    #[instrument(level = "trace")]
    async fn action_at_priority(
        &mut self,
        priority: &ActionPriority,
        game: &mut ONUWGame,
        player: &GamePlayer,
    ) -> Result<(), String> {
        Err("no actions defined for {:?}".to_string())
    }

    #[instrument(level = "trace")]
    fn after_vote(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        votes: &HashMap<GamePlayer, GamePlayer>,
        dead: &[GamePlayer],
    ) -> Vec<ONUWGameVoteAction> {
        Vec::new()
    }

    fn win_condition(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        dead: &[(GamePlayer, GameRole)],
    ) -> bool;

    #[instrument(level = "trace", skip(winners))]
    fn prevent_win<'a>(
        &self,
        game: &ONUWGame,
        player: &GamePlayer,
        #[allow(unused)] winners: &'a [(&'a dyn PlayerInterface, &'a dyn Role)],
        votes: &'a HashMap<GamePlayer, GamePlayer>,
    ) -> Vec<&'a dyn PlayerInterface> {
        Vec::new()
    }
}

dyn_clone::clone_trait_object!(Role);

impl<'a> Display for &'a dyn Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.verbose_id())
    }
}

impl<'a> PartialEq for &'a dyn Role {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
    }
}

impl<'a> PartialOrd for &'a dyn Role {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for &'a dyn Role {}

impl<'a> Ord for &'a dyn Role {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a> Hash for &'a dyn Role {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state)
    }
}
