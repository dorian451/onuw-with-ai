pub(crate) mod request;
pub(crate) mod response;

pub mod error;

use self::{
    error::{AgentError, AgentResult},
    request::Request,
    response::Response,
};
use crate::agent::{Agent, AgentChannelItem};
use async_trait::async_trait;
use onuw_game::{
    game::time::ONUWTime,
    playerinterface::{
        error::PlayerInterfaceError, message::Message, roletarget::RoleTarget, PlayerInterface,
    },
    role::{roletype::RoleType, Role},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::UnboundedSender,
    oneshot::{self},
};
use tracing::instrument;

#[derive(Debug)]
pub struct AgentInterface {
    name: String,
    agent: UnboundedSender<AgentChannelItem>,
}

impl AgentInterface {
    pub fn new(name: String) -> AgentResult<Self> {
        Ok(Self {
            name,
            agent: Agent::init()?,
        })
    }

    async fn send(&self, req: Request) -> AgentResult<Option<Response>> {
        let (tx, rx) = oneshot::channel();
        self.agent
            .send((tx, req))
            .map_err(|err| AgentError::CommunicationError {
                error: err.to_string(),
            })?;

        rx.await.map_err(|err| AgentError::CommunicationError {
            error: err.to_string(),
        })?
    }
}

#[async_trait]
impl PlayerInterface for AgentInterface {
    #[instrument(level = "trace")]
    fn name(&self) -> &str {
        self.name.as_str()
    }

    #[instrument(level = "trace")]
    async fn show_role(
        &self,
        target: RoleTarget,
        role: &dyn Role,
    ) -> Result<(), PlayerInterfaceError> {
        match self
            .send(Request::ShowRole(target, role.effective_id()))
            .await?
        {
            None => Ok(()),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn show_role_type(
        &self,
        target: RoleTarget,
        role: &RoleType,
    ) -> Result<(), PlayerInterfaceError> {
        match self
            .send(Request::ShowRoleType(target, role.to_string()))
            .await?
        {
            None => Ok(()),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn choose_player<'a>(
        &self,
        players: &'a [&'a Arc<dyn PlayerInterface>],
    ) -> Result<Arc<dyn PlayerInterface>, PlayerInterfaceError> {
        match self
            .send(Request::ChoosePlayer(
                players.iter().map(|v| v.to_owned().to_owned()).collect(),
            ))
            .await?
        {
            Some(Response::Player(p)) => Ok(p),
            None => Err(PlayerInterfaceError::UnexpectedResponse(
                "empty".to_string(),
            )),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn choose_bool(&self) -> Result<bool, PlayerInterfaceError> {
        match self.send(Request::ChooseBool()).await? {
            Some(Response::Bool(b)) => Ok(b),
            None => Err(PlayerInterfaceError::UnexpectedResponse(
                "empty".to_string(),
            )),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn choose_num(&self, choices: &[isize]) -> Result<isize, PlayerInterfaceError> {
        match self.send(Request::ChooseNum(choices.into())).await? {
            Some(Response::Num(n)) => Ok(n),
            None => Err(PlayerInterfaceError::UnexpectedResponse(
                "empty".to_string(),
            )),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn receive_message(&self, message: &Message) -> Result<(), PlayerInterfaceError> {
        match self.send(Request::ShowMessage(message.clone())).await? {
            None => Ok(()),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn handshake<'a>(
        &self,
        players: &'a [&'a Arc<dyn PlayerInterface>],
        roles: &'a HashMap<String, usize>,
    ) -> Result<(), PlayerInterfaceError> {
        match self
            .send(Request::Initialize {
                players: players.iter().map(|v| v.to_owned().to_owned()).collect(),
                roles: roles.to_owned(),
            })
            .await?
        {
            None => Ok(()),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn show_time(&self, time: &ONUWTime) -> Result<(), PlayerInterfaceError> {
        match self.send(Request::ShowTime(time.into())).await? {
            None => Ok(()),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }

    #[instrument(level = "trace")]
    async fn show_win(&self, won_game: bool) -> Result<(), PlayerInterfaceError> {
        match self.send(Request::ShowWin(won_game)).await? {
            None => Ok(()),
            Some(r) => Err(PlayerInterfaceError::UnexpectedResponse(format!("{:?}", r))),
        }
    }
}
