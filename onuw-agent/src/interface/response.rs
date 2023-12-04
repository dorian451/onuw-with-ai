use onuw_game::game::GamePlayer;

#[derive(Debug)]
pub enum Response {
    Player(GamePlayer),
    Bool(bool),
    Num(isize),
}
