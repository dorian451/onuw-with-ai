use super::{GamePlayer, GameRole};

#[derive(Clone, Debug)]
pub enum NightAction {
    Real(GameRole, GamePlayer),
    Fake(GameRole),
}
