use crate::role::Role;

#[derive(Debug)]
pub enum ONUWTime<'a> {
    Dusk,
    Night(&'a dyn Role),
    Day,
    Vote,
}
