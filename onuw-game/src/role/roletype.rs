use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RoleType {
    Villager,
    Werewolf,
    Other(&'static str),
}

impl Display for RoleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
