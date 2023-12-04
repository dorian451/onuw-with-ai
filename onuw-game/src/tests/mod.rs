mod testplayer;

use self::testplayer::TestPlayerInterface;
use crate::{
    game::{options::Options, ONUWGame},
    playerinterface::{
        message::{ClaimType, Message, MessageType, QuestionType},
        PlayerInterface,
    },
    role::{
        roles::{
            doppelganger::Doppelganger, drunk::Drunk, hunter::Hunter, insomniac::Insomniac,
            mason::Mason, minion::Minion, robber::Robber, seer::Seer, tanner::Tanner,
            troublemaker::Troublemaker, villager::Villager, werewolf::Werewolf,
        },
        Role,
    },
    tests::testplayer::Response,
};
use std::{collections::BTreeMap, future::ready, sync::Arc};
use testplayer::TestPlayer;
use tracing::{error, info, warn};
use tracing_subscriber::{filter::LevelFilter, fmt, fmt::format, fmt::format::FmtSpan, EnvFilter};

fn init_logging() {
    fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .event_format(format().pretty().without_time())
        .try_init()
        .unwrap_or_default();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{}", panic_info);
        prev_hook(panic_info);
    }));
}

fn players(num: i32) -> Vec<Arc<TestPlayerInterface>> {
    (0..num)
        .map(|i| Arc::new(TestPlayer::init(format!("{}", i))))
        .collect()
}

#[tokio::test]
async fn test() {
    init_logging();

    let players = players(11);
    let choices = vec![
        vec![
            Response::Player(players.get(4).unwrap().clone()),
            Response::Player(players.get(9).unwrap().clone()),
        ],
        vec![Response::Player(players.get(9).unwrap().clone())],
        vec![Response::Player(players.get(9).unwrap().clone())],
        vec![Response::Player(players.get(9).unwrap().clone())],
        vec![Response::Player(players.get(9).unwrap().clone())],
        vec![Response::Player(players.get(9).unwrap().clone())],
        vec![
            Response::Bool(false),
            Response::Num(0),
            Response::Num(1),
            Response::Player(players.get(9).unwrap().clone()),
        ],
        vec![
            Response::Bool(true),
            Response::Player(players.get(10).unwrap().clone()),
            Response::Player(players.get(9).unwrap().clone()),
        ],
        vec![
            Response::Bool(true),
            Response::Player(players.first().unwrap().clone()),
            Response::Player(players.get(1).unwrap().clone()),
            Response::Player(players.get(9).unwrap().clone()),
        ],
        vec![
            // Response::Num(2),
            Response::Player(players.get(2).unwrap().clone()),
        ],
        vec![Response::Player(players.get(1).unwrap().clone())],
    ];

    for (p, mut c) in players.iter().zip(choices.into_iter()) {
        c.reverse();
        p.push_choice(c).await.unwrap();
    }

    let mut game = ONUWGame::new(
        players
            .into_iter()
            .map(|v| v as Arc<dyn PlayerInterface>)
            .collect(),
        vec![
            Box::new(Doppelganger::new()),
            Box::new(Werewolf::new()),
            Box::new(Werewolf::new()),
            Box::new(Minion::new()),
            Box::new(Mason::new()),
            Box::new(Mason::new()),
            Box::new(Seer::new()),
            Box::new(Robber::new()),
            Box::new(Troublemaker::new()),
            Box::new(Tanner::new()),
            Box::new(Insomniac::new()),
            Box::new(Villager::new()),
            Box::new(Drunk::new()),
            Box::new(Hunter::new()),
        ],
        Options::new().debug_with_set_roles(),
    )
    .await
    .unwrap();

    game.send_handshake().await;
    game.show_all_roles().await;

    while let Some(priority) = game.peek_next_night_action() {
        warn!("doing night action {:#>}", priority);
        game.perform_next_night_action(|| ready(())).await.unwrap();
    }

    info!(
        "game state:\n{:#?}",
        game.players().iter().collect::<BTreeMap<_, _>>()
    );

    warn!("voting");
    game.determine_game().await;
}

#[tokio::test]
async fn message_passing() {
    init_logging();

    let players = players(5);

    let game = ONUWGame::new(
        players
            .into_iter()
            .map(|v| v as Arc<dyn PlayerInterface>)
            .collect(),
        vec![
            Box::new(Villager::new()),
            Box::new(Villager::new()),
            Box::new(Villager::new()),
            Box::new(Villager::new()),
            Box::new(Villager::new()),
            Box::new(Villager::new()),
            Box::new(Villager::new()),
            Box::new(Villager::new()),
        ],
        Options::new().debug_with_set_roles(),
    )
    .await
    .unwrap();

    let mut players: Vec<_> = game.players().iter().map(|(p, _)| p).collect();
    players.sort_by(|a, b| a.name().cmp(b.name()));

    let example_question = Message {
        mtype: MessageType::Question(
            players[1].clone(),
            QuestionType::AreRole("Doppelganger".to_string()),
        ),
        sender: players[0].clone(),
    };

    let example_answer = Message {
        mtype: MessageType::ClaimNot(ClaimType::IsRole("Doppelganger".to_string())),
        sender: players[1].clone(),
    };

    warn!("sending message 1");
    game.send_message_to_players(example_question).await;

    warn!("sending message 2");
    game.send_message_to_players(example_answer).await;
}
