use crate::core::decider;
use crate::core::imgsource::ImageSource;

use std::{env, sync::Arc};

use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::update_listeners::webhooks;
use tracing::instrument;

pub async fn run() -> Result<()> {
    let bot = Bot::from_env();

    let bot_url = env::var("BOT_URL")
        .expect("BOT_URL not set")
        .parse()
        .expect("BOT_URL is in incorrect format");

    let bot_port = env::var("BOT_PORT")
        .expect("BOT_PORT not set")
        .parse::<u16>()
        .expect("BOT_PORT is not a number");

    let listener = webhooks::axum(
        bot.clone(),
        webhooks::Options::new(([0, 0, 0, 0], bot_port).into(), bot_url),
    )
    .await
    .expect("Webhook creation failed");

    let source = Arc::new(ImageSource::new().await?);

    Box::pin(teloxide::repl_with_listener(
        bot,
        move |bot: Bot, msg: Message| {
            let source = source.clone();
            async move {
                handle_message(bot.clone(), msg, source).await;
                Ok(())
            }
        },
        listener,
    ))
    .await;

    Ok(())
}

#[instrument(skip(bot, msg, source), fields(chat_id = %msg.chat.id))]
async fn handle_message(bot: Bot, msg: Message, source: Arc<ImageSource>) {
    tracing::info!("Message recieved from {}", &msg.chat.id);

    if let Some(text) = msg.text() {
        if !decider::should_respond(text) {
            tracing::info!("Decided to not respond");
            return;
        }
    } else {
        tracing::info!("Not a text mesasge, ignoring");
        return;
    }

    tracing::info!("Looking for image");

    match source.get_image().await {
        Err(err) => {
            tracing::error!("Failed to get image {:?}", err);
        }
        Ok(image) => {
            let send_photo_result = bot.send_photo(msg.chat.id, InputFile::memory(image)).await;

            if let Err(err) = send_photo_result {
                tracing::warn!("Failed to send message: {:?}", err);
            } else {
                tracing::info!("Photo sent");
            }
        }
    }
}
