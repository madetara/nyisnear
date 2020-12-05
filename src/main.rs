mod core;

use crate::core::*;

extern crate pretty_env_logger;
use std::{
    env,
    net::{IpAddr, Ipv4Addr},
};
use tbot::{prelude::*, types::input_file::Photo};

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let mut bot = tbot::Bot::from_env("BOT_TOKEN").event_loop();

    bot.text(|context| async move {
        log::info!("Message recieved from {}", &context.chat.id);

        if !decider::should_respond(context.text.value.as_str()).await {
            return;
        }

        match imgsource::get_image().await {
            Err(err) => {
                log::error!("Failed to get image: {:?}", err);
            }
            Ok(image) => {
                let call_result = context
                    .send_photo_in_reply(Photo::with_bytes(image.as_ref()))
                    .call()
                    .await;

                if let Err(err) = call_result {
                    log::warn!("Failed to send message: {:?}", err);
                } else {
                    log::info!("Message sent");
                }
            }
        }
    });

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
