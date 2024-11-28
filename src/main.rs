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
    ReceiveCode(account::Account),
    Onboarding(account::Account),
    SendLocation(account::Account),
    Chat(account::Account),
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

async fn establish_connection() -> Result<sqlx::PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url).await?;

    Ok(pool)
}

type Storage = RedisStorage<Json>;
type Dialogue = teloxide::dispatching::dialogue::Dialogue<StateMachine, Storage>;
type HandlerResult = Result<(), anyhow::Error>;

async fn authentication(bot: Bot, dialogue: Dialogue, msg: Message) -> HandlerResult {
    let pool = establish_connection()
        .await
        .expect("Failed to connect to database");

    let chat_id = msg.chat.id.0;
    let account = account::Account::find_by_chat_id(&chat_id, &pool).await?;

    if let Some(account) = account {
        if account.verified {
            if account.did_onboarding {
                dialogue.update(StateMachine::Chat(account)).await?;
                return Ok(());
            }

            dialogue.update(StateMachine::Onboarding(account)).await?;
            return Ok(());
        }
    }

    let first_name = if let Some(first_name) = msg.chat.first_name() {
        first_name.to_string()
    } else {
        "".to_string()
    };

    let last_name = if let Some(last_name) = msg.chat.last_name() {
        last_name.to_string()
    } else {
        "".to_string()
    };

    let account = account::Account {
        chat_id: chat_id,
        first_name: first_name,
        last_name: last_name,
        ..Default::default()
    };

    account.create(&pool).await?;

    let message = format!(
        "Olá, {}! Para continuar, precisamos verificar sua identidade. Por favor, digite o código que você recebeu.",
        account.first_name
    );

    bot.send_message(msg.chat.id, message).await?;
    dialogue.update(StateMachine::ReceiveCode(account)).await?;

    Ok(())
}

async fn receive_code(
    bot: Bot,
    dialogue: Dialogue,
    mut account: account::Account,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    match msg.text() {
        Some(code) => {
            if code == "1234" {
                let pool = establish_connection()
                    .await
                    .expect("Failed to connect to database");

                account.verified = true;
                account.update(&pool).await?;
                dialogue.update(StateMachine::Onboarding(account)).await?;
            } else {
                bot.send_message(chat_id, "Código incorreto!").await?;
            }
        }
        None => {
            bot.send_message(chat_id, "Por favor, digite o código que você recebeu.")
                .await?;
        }
    }

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
    let pool = establish_connection()
        .await
        .expect("Failed to connect to database");

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, Storage, StateMachine>()
            .branch(dptree::case![StateMachine::Authentication].endpoint(authentication))
            .branch(dptree::case![StateMachine::ReceiveCode(account)].endpoint(receive_code)),
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
