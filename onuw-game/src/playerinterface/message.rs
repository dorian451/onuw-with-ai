use crate::game::GamePlayer;

#[derive(Clone, Debug)]
pub struct Message {
    pub mtype: MessageType,
    pub sender: GamePlayer,
}

#[derive(Clone, Debug)]
pub enum MessageType {
    Claim(ClaimType),
    ClaimNot(ClaimType),
    Question(GamePlayer, QuestionType),
}

#[derive(Clone, Debug)]
pub enum ClaimType {
    IsRole(String),
    PerformedRoleActionToSelf(String),
    PerformedRoleActionToOne(String, GamePlayer),
    PerformedRoleActionToTwo(String, GamePlayer, GamePlayer),
}

#[derive(Clone, Debug)]
pub enum QuestionType {
    WhatRole,
    AreRole(String),
    DidRoleActionToSelf(String),
    DidRoleActionToOne(String, GamePlayer),
    DidRoleActionToTwo(String, GamePlayer, GamePlayer),
}
