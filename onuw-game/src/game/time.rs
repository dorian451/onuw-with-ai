use crate::role::Role;

use super::GamePlayer;

#[derive(Debug)]
pub enum ONUWTime<'a> {
    Dusk,
    Night(&'a dyn Role),
    Day,
    Vote,
    End {
        dead: &'a [GamePlayer],
        winners: &'a [GamePlayer],
    },
}
