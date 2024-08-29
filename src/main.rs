use std::{env, fs::File, io::BufReader};

use actix_web::{
    rt,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_ws::AggregatedMessage;
use dotenv::dotenv;
use futures_util::StreamExt as _;
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{self, ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole},
    common::GPT4_O_MINI,
};

async fn chat(
    req: HttpRequest,
    stream: web::Payload,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        let mut messages: Vec<ChatCompletionMessage> = vec![];
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let user_message: ChatCompletionMessage = serde_json::from_str(&text).unwrap();
                    messages.push(user_message);
                    let req = ChatCompletionRequest::new(GPT4_O_MINI.to_string(), messages.clone());
                    let res = &state.openai_client.chat_completion(req).await.unwrap();

                    let choices = &res.choices;
                    let assistant_message = &choices[0].message;
                    let assistant_message = ChatCompletionMessage {
                        role: MessageRole::assistant,
                        content: Content::Text(assistant_message.content.to_owned().unwrap()),
                        name: None,
                        tool_call_id: None,
                        tool_calls: None,
                    };
                    messages.push(assistant_message.clone());
                    let text = serde_json::to_string(&assistant_message).unwrap();

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

    Ok(res)
}

struct AppState {
    openai_client: OpenAIClient,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    // let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    // let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    // let tls_certs = rustls_pemfile::certs(&mut certs_file)
    //     .collect::<Result<Vec<_>, _>>()
    //     .unwrap();
    // let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
    //     .next()
    //     .unwrap()
    //     .unwrap();

    // let tls_config = rustls::ServerConfig::builder()
    //     .with_no_client_auth()
    //     .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
    //     .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                openai_client: OpenAIClient::new(env::var("OPENAI_API_KEY").unwrap().to_string()),
            }))
            .route("/ws", web::get().to(chat))
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
