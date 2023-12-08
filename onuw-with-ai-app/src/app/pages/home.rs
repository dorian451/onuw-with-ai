use crate::app::components::startgame::GameConfig;
use fallible_iterator::{FallibleIterator, IteratorExt};
use leptos::*;
use leptos_router::*;
use onuw_game::{
    game::{options::Options, ONUWGame},
    playerinterface::PlayerInterface,
    role::{
        roles::{RoleDef, ROLES, ROLES_STRINGS},
        Role,
    },
};
use std::time::Duration;
use std::{iter::repeat_with, sync::Arc};
use tracing::info;

#[cfg(feature = "ssr")]
use onuw_agent::interface::{error::AgentError, AgentInterface};
#[cfg(feature = "ssr")]
use tokio::runtime::Handle;

#[derive(Clone, Copy, Debug)]
enum GameType {
    Mixed,
    AiOnly,
}

#[component]
pub fn HomePage() -> impl IntoView {
    let btn_active = create_rw_signal(None);

    let start_game_action = create_action(|x: &Vec<(StoredValue<RoleDef>, usize)>| {
        run_game(
            x.iter()
                .map(|(v, amt)| (v.get_value().name, *amt))
                .collect(),
        )
    });

    create_effect(move |_| info!("active: {:?}", btn_active()));

    view! {
        <div class="min-w-fit flex flex-col gap-5 items-center">
            <h1 class="text-center">"Welcome to One Night Ultimate Werewolf with AI!"</h1>

            <button class="bg-green-500 dark:bg-green-700" disabled on:click=|_| info!("hello")>
                "Start new game"
            </button>

            <button
                class="bg-slate-500 dark:bg-slate-700"
                on:click=move |_| {
                    btn_active
                        .update(move |v: &mut _| {
                            *v = if v.is_some() { None } else { Some(GameType::AiOnly) };
                        })
                }
            >

                "Start new game with only AI agents"
            </button>

            {move || {
                if btn_active().is_some() {
                    Some(
                        view! {
                            <GameConfig
                                new_game_label="Start new AI-only game"
                                on_click=Callback::new(move |x| {
                                    start_game_action.dispatch(x);
                                    btn_active.set(None);
                                })
                            />
                        },
                    )
                } else {
                    None
                }
            }}

            {move || {
                start_game_action
                    .value()
                    .get()
                    .map(|v| {
                        if let Ok((votes, dead, winners, roles)) = v {
                            Some(
                                view! {
                                    <p>{move || format!("Votes: {:#?}", votes)}</p>
                                    <p>{move || format!("Dead: {:#?}", dead)}</p>
                                    <p>{move || format!("Winners: {:#?}", winners)}</p>
                                    <p>{move || format!("Roles: {:#?}", roles)}</p>
                                },
                            )
                        } else {
                            None
                        }
                    })
            }}

        </div>
    }
}

#[server]
async fn run_game(
    requested_roles: Vec<(String, usize)>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>), ServerFnError> {
    let roles: Vec<_> = requested_roles
        .into_iter()
        .map(|(rname, amt)| {
            ROLES_STRINGS
                .get(&rname)
                .ok_or(ServerFnError::ServerError(format!(
                    "invalid role: {}",
                    rname
                )))
                .map(|v| (v, amt))
        })
        .transpose_into_fallible()
        .map(|(rdef, amt)| {
            if (rdef.min_amt..=rdef.max_amt).contains(&amt) {
                Ok(repeat_with(|| Ok(ROLES.get(rdef).unwrap().unwrap()()))
                    .take(amt)
                    .transpose_into_fallible())
            } else {
                Err(ServerFnError::ServerError(format!(
                    "wrong amount of {}",
                    rdef.name
                )))
            }
        })
        .flatten()
        .collect()?;

    info!("Starting new game with roles: {:#?}", roles);

    let mut count = 0;

    let players: Vec<_> = repeat_with(|| {
        count += 1;
        Ok::<_, AgentError>(
            Arc::new(AgentInterface::new(format!("AI Agent {}", count))?)
                as Arc<dyn PlayerInterface>,
        )
    })
    .take(roles.len() - 3)
    .transpose_into_fallible()
    .collect()?;

    let mut game = ONUWGame::new(players, roles, Options::default()).await?;

    game.send_handshake().await;

    game.show_all_roles().await;

    while game.peek_next_night_action().is_some() {
        game.perform_next_night_action(|| async {}).await?;
    }

    game.collect_votes().await?;

    game.calc_dead_and_winners().await?;

    Ok((
        game.votes()
            .as_ref()
            .unwrap()
            .iter()
            .map(|(p, v)| format!("{}: {}", p.name(), v.name()))
            .collect(),
        game.dead()
            .as_ref()
            .unwrap()
            .clone()
            .into_iter()
            .map(|v| v.name().to_string())
            .collect(),
        game.winners()
            .as_ref()
            .unwrap()
            .clone()
            .into_iter()
            .map(|v| v.name().to_string())
            .collect(),
        game.players()
            .iter()
            .map(|(p, v)| format!("{}: {}", p.name(), v.try_read().unwrap().verbose_id()))
            .collect(),
    ))
    // Ok(([].to_vec(), [].to_vec()))
}
