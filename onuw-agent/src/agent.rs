use std::collections::HashMap;

use crate::{
    interface::{
        error::{AgentError, AgentResult},
        request::Request,
        response::Response,
    },
    roles::Role as KnownRole,
};
use fallible_iterator::{FallibleIterator, IteratorExt};
use onuw_game::game::GamePlayer;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};
use tracing::warn;

pub type AgentChannelItem = (oneshot::Sender<AgentResult<Option<Response>>>, Request);

#[derive(Debug)]
pub struct Agent {
    other_players: Vec<GamePlayer>,
    possible_roles: HashMap<KnownRole, usize>,
}

impl Agent {
    fn new(players: Vec<GamePlayer>, roles: HashMap<String, usize>) -> AgentResult<Self> {
        let possible_roles = roles
            .into_iter()
            .map(|(k, v)| Ok((KnownRole::try_from(k.as_str())?, v)))
            .transpose_into_fallible()
            .collect()?;

        Ok(Self {
            other_players: players,
            possible_roles,
        })
    }

    pub fn init() -> AgentResult<UnboundedSender<AgentChannelItem>> {
        let (tx, mut rx) = mpsc::unbounded_channel::<AgentChannelItem>();

        tokio::spawn(async move {
            let self_;

            loop {
                let req = rx.recv().await;

                if let Some((sender, Request::Initialize { players, roles })) = req {
                    match Self::new(players, roles) {
                        Ok(r) => {
                            self_ = r;
                            sender.send(Ok(None)).unwrap();
                            break;
                        }
                        Err(e) => {
                            sender.send(Err(e)).unwrap();
                        }
                    }
                } else if let Some((sender, _)) = req {
                    sender.send(Err(AgentError::UninitializedError)).unwrap();
                }
            }

            while let Some((sender, req)) = rx.recv().await {
                match req {
                    Request::ChoosePlayer(_) => todo!(),
                    Request::ChooseBool() => todo!(),
                    Request::ChooseNum(_) => todo!(),
                    Request::ShowMessage(_) => todo!(),
                    Request::ShowWin(_) => todo!(),
                    Request::ShowTime(_) => todo!(),
                    Request::Initialize { .. } => {
                        warn!("Agent already initialized!");
                    }
                    Request::ShowRole(_, _) => todo!(),
                    Request::ShowRoleType(_, _) => todo!(),
                }
            }
        });

        Ok(tx)
    }
}
