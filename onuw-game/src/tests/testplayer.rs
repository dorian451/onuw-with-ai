use crate::{
    game::time::ONUWTime,
    playerinterface::{
        error::PlayerInterfaceError, message::Message, roletarget::RoleTarget, PlayerInterface,
    },
    role::{Role, roletype::RoleType},
};
use async_trait::async_trait;
use futures::{
    channel::{
        mpsc::{unbounded, UnboundedSender},
        oneshot,
    },
    StreamExt,
};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument, warn};

enum Request {
    PushChoice(Vec<Response>),
    ChoosePlayer(Vec<Arc<dyn PlayerInterface>>),
    ChooseBool(),
    ChooseNum(Vec<isize>),
    ShowMessage(Message),
}

#[derive(Debug)]
pub enum Response {
    Player(Arc<dyn PlayerInterface>),
    Bool(bool),
    Num(isize),
}

#[derive(Debug)]
pub struct TestPlayer {
    name: String,
    choices: Vec<Response>,
}

impl TestPlayer {
    pub fn init(name: String) -> TestPlayerInterface {
        let (tx, mut rx) =
            unbounded::<(oneshot::Sender<Result<Option<Response>, String>>, Request)>();

        let name2 = name.clone();

        let handle = tokio::spawn(async move {
            let mut self_ = Self {
                name: name2,
                choices: Vec::new(),
            };

            while let Some((tx1, req)) = rx.next().await {
                tx1.send(match req {
                    Request::PushChoice(c) => {
                        self_.push_choice(c);
                        Ok(None)
                    }
                    Request::ChoosePlayer(players) => self_
                        .choose_player(players)
                        .map(|v| Some(Response::Player(v))),
                    Request::ChooseBool() => self_.choose_bool().map(|v| Some(Response::Bool(v))),
                    Request::ChooseNum(range) => {
                        self_.choose_num(range).map(|v| Some(Response::Num(v)))
                    }
                    Request::ShowMessage(message) => {
                        info!("Player {} received message: {:?}", self_.name, message);
                        Ok(None)
                    }
                })
                .unwrap();
            }
        });

        TestPlayerInterface {
            name,
            send: tx,
            _handle: handle,
        }
    }

    #[instrument(level = "trace")]
    fn push_choice(&mut self, mut c: Vec<Response>) {
        self.choices.append(&mut c);
    }

    #[instrument(level = "trace")]
    fn choose_player(
        &mut self,
        mut players: Vec<Arc<dyn PlayerInterface>>,
    ) -> Result<Arc<dyn PlayerInterface>, String> {
        match self.choices.pop() {
            Some(Response::Player(player)) => {
                if players.contains(&player) {
                    Ok(player)
                } else {
                    players.sort();

                    Err(format!(
                        "Invalid player choice {:#?} from {:#?}",
                        player, players
                    ))
                }
            }
            Some(r) => Err(format!(
                "wrong choice type! was expecting Player, got {:?}",
                r
            )),
            None => Err("ran out of choices!".to_string()),
        }
    }

    #[instrument(level = "trace")]
    fn choose_num(&mut self, range: Vec<isize>) -> Result<isize, String> {
        match self.choices.pop() {
            Some(Response::Num(num)) => {
                if range.contains(&num) {
                    Ok(num)
                } else {
                    Err(format!("choice {} out of range {:?}", num, range))
                }
            }
            Some(r) => Err(format!("wrong choice type! was expecting Num, got {:?}", r)),
            None => Err("ran out of choices!".to_string()),
        }
    }

    #[instrument(level = "trace")]
    fn choose_bool(&mut self) -> Result<bool, String> {
        match self.choices.pop() {
            Some(Response::Bool(num)) => Ok(num),
            Some(r) => Err(format!(
                "wrong choice type! was expecting Bool, got {:?}",
                r
            )),
            None => Err("ran out of choices!".to_string()),
        }
    }
}

pub struct TestPlayerInterface {
    name: String,
    #[allow(clippy::type_complexity)]
    send: UnboundedSender<(oneshot::Sender<Result<Option<Response>, String>>, Request)>,
    _handle: JoinHandle<()>,
}

impl Debug for TestPlayerInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestPlayerInterface")
            .field("name", &self.name)
            .finish()
    }
}

impl TestPlayerInterface {
    pub async fn push_choice(&self, c: Vec<Response>) -> Result<(), PlayerInterfaceError> {
        let (tx, rx) = oneshot::channel();
        self.send
            .unbounded_send((tx, Request::PushChoice(c)))
            .unwrap();

        let resp = rx.await;
        if let Ok(Ok(None)) = resp {
            Ok(())
        } else {
            Err(PlayerInterfaceError::CommunicationError(format!(
                "error pushing choice {:#?}",
                resp,
            )))
        }
    }
}

#[async_trait]
impl PlayerInterface for TestPlayerInterface {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    #[instrument(level = "trace", skip(role))]
    async fn show_role(
        &self,
        target: RoleTarget,
        role: &dyn Role,
    ) -> Result<(), PlayerInterfaceError> {
        info!(
            "Player {} was shown {:?}'s role of {:?}",
            self.name(),
            target,
            role
        );
        Ok(())
    }

    #[instrument(level = "trace")]
    async fn show_win(&self, won_game: bool) -> Result<(), PlayerInterfaceError> {
        info!(
            "Player {} was shown that they {} the game.",
            self.name(),
            if won_game { "won" } else { "lost" }
        );

        Ok(())
    }

    #[instrument(level = "trace")]
    async fn show_role_type(
        &self,
        target: RoleTarget,
        role_type: &RoleType,
    ) -> Result<(), PlayerInterfaceError> {
        info!(
            "Player {} was shown {:?}'s role type of {:?}",
            self.name(),
            target,
            role_type
        );
        Ok(())
    }

    #[instrument(level = "trace")]
    async fn choose_player<'a>(
        &self,
        players: &'a [&'a Arc<dyn PlayerInterface>],
    ) -> Result<Arc<dyn PlayerInterface>, PlayerInterfaceError> {
        let (tx, rx) = oneshot::channel();

        self.send
            .unbounded_send((
                tx,
                Request::ChoosePlayer(players.iter().copied().cloned().collect()),
            ))
            .unwrap();

        let resp = rx.await;
        if let Ok(Ok(Some(Response::Player(player)))) = resp {
            Ok(player)
        } else {
            Err(PlayerInterfaceError::CommunicationError(format!(
                "error choosing player: {:#?}",
                resp,
            )))
        }
    }

    #[instrument(level = "trace")]
    async fn choose_bool(&self) -> Result<bool, PlayerInterfaceError> {
        let (tx, rx) = oneshot::channel();

        self.send
            .unbounded_send((tx, Request::ChooseBool()))
            .unwrap();

        let resp = rx.await;
        if let Ok(Ok(Some(Response::Bool(choice)))) = resp {
            Ok(choice)
        } else {
            Err(PlayerInterfaceError::CommunicationError(format!(
                "error choosing bool {:#?}",
                resp
            )))
        }
    }

    #[instrument(level = "trace")]
    async fn choose_num(&self, choices: &[isize]) -> Result<isize, PlayerInterfaceError> {
        let (tx, rx) = oneshot::channel();

        self.send
            .unbounded_send((tx, Request::ChooseNum(choices.to_owned())))
            .unwrap();

        let resp = rx.await;
        if let Ok(Ok(Some(Response::Num(choice)))) = resp {
            Ok(choice)
        } else {
            Err(PlayerInterfaceError::CommunicationError(format!(
                "error choosing bool {:#?}",
                resp
            )))
        }
    }

    #[instrument(level = "trace")]
    async fn handshake<'a>(
        &self,
        players: &'a [&'a Arc<dyn PlayerInterface>],
        roles: &'a HashMap<String, usize>,
    ) -> Result<(), PlayerInterfaceError> {
        debug!(
            "Player {} was greeted by {:?} with {:?}",
            self.name(),
            players,
            roles
        );
        Ok(())
    }

    #[instrument(level = "trace")]
    async fn show_time(&self, time: &ONUWTime) -> Result<(), PlayerInterfaceError> {
        debug!(
            "Player {} was shown that the time is {:?}",
            self.name(),
            time
        );
        Ok(())
    }

    #[instrument(level = "trace")]
    async fn receive_message(&self, message: &Message) -> Result<(), PlayerInterfaceError> {
        let (tx, rx) = oneshot::channel();
        self.send
            .unbounded_send((tx, Request::ShowMessage(message.clone())))
            .unwrap();

        let resp = rx.await;
        if let Ok(Ok(None)) = resp {
            Ok(())
        } else {
            Err(PlayerInterfaceError::CommunicationError(format!(
                "error receiving message {:#?}",
                resp,
            )))
        }
    }
}
