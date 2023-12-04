use onuw_game::{
    game::{time::ONUWTime, GamePlayer},
    playerinterface::{message::Message, roletarget::RoleTarget},
};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Request {
    Initialize {
        players: Vec<GamePlayer>,
        roles: HashMap<String, usize>,
    },
    ChoosePlayer(Vec<GamePlayer>),
    ChooseBool(),
    ChooseNum(Vec<isize>),
    ShowMessage(Message),
    ShowRole(RoleTarget, String),
    ShowRoleType(RoleTarget, String),
    ShowWin(bool),
    ShowTime(Time),
}

#[derive(Debug)]
pub enum Time {
    Dusk,
    Night(String),
    Day,
    Vote,
}

impl<'a> From<&ONUWTime<'a>> for Time {
    fn from(value: &ONUWTime) -> Self {
        match value {
            ONUWTime::Dusk => Time::Dusk,
            ONUWTime::Night(r) => Time::Night(r.effective_id()),
            ONUWTime::Day => Time::Day,
            ONUWTime::Vote => Time::Vote,
        }
    }
}
