use actix_web::{
    get, rt,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_ws::AggregatedMessage;
use anyhow;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::serializer::Json, dialogue::RedisStorage, HandlerExt},
    prelude::*,
    utils::command::BotCommands,
};
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct State {
    bot: Bot,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum StateMachine {
    #[default]
    Authentication,
    Onboarding,
    Chat,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
    };

    Ok(())
}

#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    state: Data<State>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    let bot = state.bot.clone();

                    session.text(text).await.unwrap();
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary message
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}

type Storage = RedisStorage<Json>;
type Dialogue = teloxide::dispatching::dialogue::Dialogue<StateMachine, Storage>;
type HandlerResult = Result<(), anyhow::Error>;

async fn authentication(bot: Bot, dialogue: Dialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! What's your full name?")
        .await?;
    dialogue.update(StateMachine::Onboarding).await?;
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting application");
    let state = State {
        bot: Bot::from_env(),
    };

    let bot = state.bot.clone();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, Storage, StateMachine>()
            .branch(dptree::case![StateMachine::Authentication].endpoint(authentication)),
    )
    .dependencies(dptree::deps![RedisStorage::open(
        "redis://127.0.0.1/",
        Json,
    )
    .await
    .expect("Failed to open Redis storage")])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .service(websocket)
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
