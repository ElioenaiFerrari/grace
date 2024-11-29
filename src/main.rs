use anyhow;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{dialogue::serializer::Json, dialogue::RedisStorage, HandlerExt},
    prelude::*,
    utils::command::BotCommands,
};
mod account;
mod agent;
mod message;

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
    ReceiveLocation(account::Account),
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

            dialogue.update(StateMachine::ReceiveCode(account)).await?;
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

                let message = format!(
                    r#"
                    Olá, {}! Seja bem-vindo ao nosso bot. Para continuar, precisamos de sua localização.
                "#,
                    account.first_name
                );

                bot.send_message(chat_id, message).await?;

                dialogue.update(StateMachine::Chat(account)).await?;
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

// async fn onboarding(
//     bot: Bot,
//     dialogue: Dialogue,
//     mut account: account::Account,
//     msg: Message,
// ) -> HandlerResult {
//     let chat_id = msg.chat.id;

//     dialogue
//         .update(StateMachine::ReceiveLocation(account))
//         .await?;

//     Ok(())
// }

async fn receive_location(
    bot: Bot,
    dialogue: Dialogue,
    mut account: account::Account,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;

    match msg.location() {
        Some(location) => {
            let pool = establish_connection()
                .await
                .expect("Failed to connect to database");

            log::info!("Location: {:#?}", location);

            account.did_onboarding = true;
            account.update(&pool).await?;
            dialogue.update(StateMachine::Chat(account)).await?;
        }
        None => {
            bot.send_message(chat_id, "Por favor, compartilhe sua localização.")
                .await?;
        }
    }

    Ok(())
}

async fn chat(
    bot: Bot,
    dialogue: Dialogue,
    account: account::Account,
    msg: Message,
) -> HandlerResult {
    let agent = agent::Agent::default();
    let chat_id = msg.chat.id;

    match msg.text() {
        Some(text) => {
            let pool = establish_connection()
                .await
                .expect("Failed to connect to database");

            let user_message = message::Message {
                chat_id: chat_id.0,
                content: text.to_string(),
                ..Default::default()
            };
            user_message.create(&pool).await?;

            let messages = message::Message::list_by_chat_id(chat_id.0, 10, &pool).await?;

            let messages: Vec<genai::chat::ChatMessage> = messages
                .iter()
                .map(|message| -> genai::chat::ChatMessage {
                    match message.role {
                        message::Role::User => genai::chat::ChatMessage {
                            content: genai::chat::MessageContent::Text(message.content.to_string()),
                            role: genai::chat::ChatRole::User,
                        },
                        message::Role::Assistant => genai::chat::ChatMessage {
                            content: genai::chat::MessageContent::Text(message.content.to_string()),
                            role: genai::chat::ChatRole::Assistant,
                        },
                    }
                })
                .collect();

            let response = agent.send(messages).await?;
            match response.content.expect("Failed to get response") {
                genai::chat::MessageContent::Text(text) => {
                    bot.send_message(chat_id, text.clone()).await?;
                    let assistant_message = message::Message {
                        chat_id: chat_id.0,
                        content: text,
                        role: message::Role::Assistant,
                        ..Default::default()
                    };

                    assistant_message.create(&pool).await?;
                }
                _ => {
                    bot.send_message(chat_id, "Desculpe, não entendi.").await?;
                    return Ok(());
                }
            }
        }
        None => {
            bot.send_message(chat_id, "Por favor, digite uma mensagem.")
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
            .branch(dptree::case![StateMachine::ReceiveCode(account)].endpoint(receive_code))
            .branch(
                dptree::case![StateMachine::ReceiveLocation(account)].endpoint(receive_location),
            )
            .branch(dptree::case![StateMachine::Chat(account)].endpoint(chat)),
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
