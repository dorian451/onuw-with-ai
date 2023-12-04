use std::fmt::Display;

use crate::interface::error::AgentError;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) enum Role {
    Doppelganger,
    Werewolves,
    Minion,
    Masons,
    Seer,
    Robber,
    Troublemaker,
    Drunk,
    Insomniac,
    Villager,
    Hunter,
    Tanner,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> TryFrom<&'a str> for Role {
    type Error = AgentError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Doppelganger" => Ok(Self::Doppelganger),
            "Werewolves" => Ok(Self::Werewolves),
            "Minion" => Ok(Self::Minion),
            "Masons" => Ok(Self::Masons),
            "Seer" => Ok(Self::Seer),
            "Robber" => Ok(Self::Robber),
            "Troublemaker" => Ok(Self::Troublemaker),
            "Drunk" => Ok(Self::Drunk),
            "Insomniac" => Ok(Self::Insomniac),
            "Villager" => Ok(Self::Villager),
            "Hunter" => Ok(Self::Hunter),
            "Tanner" => Ok(Self::Tanner),
            role => Err(AgentError::RoleParseError {
                role: role.to_owned(),
            }),
        }
    }
}
