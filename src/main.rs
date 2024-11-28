use anyhow;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::serializer::Json, dialogue::RedisStorage, HandlerExt},
    prelude::*,
    utils::command::BotCommands,
};
mod account;

#[derive(Debug, Clone)]
pub struct State {
    bot: Bot,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum StateMachine {
    #[default]
    Authentication,
    ReceiveCode,
    Onboarding,
    SendLocation,
    Chat,
}

// #[derive(BotCommands, Clone)]
// #[command(
//     rename_rule = "lowercase",
//     description = "These commands are supported:"
// )]
// enum Command {
//     #[command(description = "display this text.")]
//     Help,
// }

// async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
//     match cmd {
//         Command::Help => {
//             bot.send_message(msg.chat.id, Command::descriptions().to_string())
//                 .await?
//         }
//     };

//     Ok(())
// }

type Storage = RedisStorage<Json>;
type Dialogue = teloxide::dispatching::dialogue::Dialogue<StateMachine, Storage>;
type HandlerResult = Result<(), anyhow::Error>;

async fn authentication(bot: Bot, dialogue: Dialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Digite o código fornecido")
        .await?;

    Ok(())
}

#[tokio::main]
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

    Ok(())
}
