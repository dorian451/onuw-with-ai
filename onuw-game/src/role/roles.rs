mod common;

pub mod doppelganger;
pub mod drunk;
pub mod hunter;
pub mod insomniac;
pub mod mason;
pub mod minion;
pub mod robber;
pub mod seer;
pub mod tanner;
pub mod troublemaker;
pub mod villager;
pub mod werewolf;

use self::doppelganger::Doppelganger;
use super::Role;
use crate::role::roles::{
    drunk::Drunk, hunter::Hunter, insomniac::Insomniac, mason::Mason, minion::Minion,
    robber::Robber, seer::Seer, tanner::Tanner, troublemaker::Troublemaker, villager::Villager,
    werewolf::Werewolf,
};
use cfg_if::cfg_if;
use hashlink::LinkedHashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{any::type_name, collections::HashMap};

type RoleInitFn = fn() -> Box<dyn Role>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct RoleDef {
    pub name: String,
    pub min_amt: usize,
    pub max_amt: usize,
}

impl RoleDef {
    fn new<T: Role + 'static>() -> Self {
        Self {
            name: type_name::<T>().split("::").last().unwrap().to_string(),
            min_amt: T::min_amt(),
            max_amt: T::max_amt(),
        }
    }

    #[cfg(not(feature = "light"))]
    fn role_as_boxed<T: Role + 'static>() -> Box<dyn Role> {
        Box::new(T::new())
    }
}

fn roledef_pair<T: Role + 'static>() -> (RoleDef, Option<RoleInitFn>) {
    cfg_if! {
        if #[cfg(feature = "light")] {
            let fun = None;
        }
        else{
            let fun: Option<RoleInitFn> = Some(RoleDef::role_as_boxed::<T>);
        }
    };

    (RoleDef::new::<T>(), fun)
}

pub static ROLES: Lazy<LinkedHashMap<RoleDef, Option<RoleInitFn>>> = Lazy::new(|| {
    [
        roledef_pair::<Doppelganger>(),
        roledef_pair::<Drunk>(),
        roledef_pair::<Hunter>(),
        roledef_pair::<Insomniac>(),
        roledef_pair::<Mason>(),
        roledef_pair::<Minion>(),
        roledef_pair::<Robber>(),
        roledef_pair::<Seer>(),
        roledef_pair::<Tanner>(),
        roledef_pair::<Troublemaker>(),
        roledef_pair::<Villager>(),
        roledef_pair::<Werewolf>(),
    ]
    .into_iter()
    .collect()
});

pub static ROLES_STRINGS: Lazy<HashMap<String, RoleDef>> = Lazy::new(|| {
    ROLES
        .iter()
        .map(|(r, _)| (r.name.clone(), r.clone()))
        .collect()
});
// pub const Roles: &[&str, ]

