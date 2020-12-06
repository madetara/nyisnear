use crate::core::decider;
use crate::core::imgsource::ImageSource;

use std::{
    env,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use anyhow::Result;
use tbot::{
    contexts::{methods::ChatMethods, Text},
    types::input_file::Photo,
};
use tracing::instrument;

pub async fn run() -> Result<()> {
    let mut bot = tbot::Bot::from_env("BOT_TOKEN").event_loop();

    let source = Arc::new(ImageSource::new().await?);

    bot.text(move |context| {
        let source = Arc::clone(&source);
        async move { handle_message(source, context).await }
    });

    start_webhook(bot).await;
    Ok(())
}

async fn start_webhook(bot: tbot::EventLoop) {
    let bot_url = env::var("BOT_URL").expect("BOT_URL not set");
    let bot_port = env::var("BOT_PORT")
        .expect("BOT_PORT not set")
        .parse::<u16>()
        .expect("BOT_PORT is not a number");

    bot.webhook(bot_url.as_str(), bot_port)
        .ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
        .http()
        .start()
        .await
        .unwrap();
}

#[instrument(skip(source, context), fields(chat_id = %context.chat.id))]
async fn handle_message(source: Arc<ImageSource>, context: Arc<Text>) {
    tracing::info!("Message recieved from {}", &context.chat.id);

    if !decider::should_respond(context.text.value.as_str()).await {
        tracing::info!("Decided to not respond");
        return;
    }

    match source.get_image().await {
        Err(err) => {
            tracing::error!("Failed to get image {:?}", err);
        }
        Ok(image) => {
            let call_result = context
                .send_photo_in_reply(Photo::with_bytes(image.as_ref()))
                .call()
                .await;

            if let Err(err) = call_result {
                tracing::warn!("Failed to send message: {:?}", err);
            } else {
                tracing::info!("Message sent");
            }
        }
    }
}
