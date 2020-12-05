use anyhow::{Context, Result};
use lazy_static::lazy_static;
use rand::Rng;
use regex::{Captures, Regex};

use crate::core::error::BotError;

pub async fn get_image() -> Result<bytes::Bytes> {
    let photo_url = get_photo_url().await.context("failed to get photo url")?;

    Ok(reqwest::get(photo_url.as_str())
        .await
        .context(format!("failed to send request to {}", photo_url))?
        .bytes()
        .await
        .map_err(|source| BotError::GetImageError { source })?)
}

const SEARCH_REQUEST: &str = "https://yandex.ru/images/search?text=%D1%85%D0%B0%D1%82%D0%B0 %D0%BD%D0%B0 %D0%BD%D0%B3 %D0%BC%D0%B5%D0%BC";

async fn get_photo_url() -> Result<String, BotError> {
    lazy_static! {
        static ref IMG_REGEX: Regex =
            Regex::new(r#""w":(\d+),"h":(\d+),"origin":\{[^\}]*(https?://[^"]*)[^<]*"#).unwrap();
    }

    let search_result = reqwest::get(SEARCH_REQUEST)
        .await
        .map_err(|source| BotError::SearchImageError { source })?
        .text()
        .await
        .map_err(|source| BotError::SearchImageError { source })?;

    let found_images = IMG_REGEX
        .captures_iter(search_result.as_str())
        .collect::<Vec<Captures>>();

    if found_images.len() <= 0 {
        return Err(BotError::ImageParseError);
    }

    let mut rnd = rand::thread_rng();
    let idx = rnd.gen_range(0, found_images.len());

    let image_url: &str = found_images
        .get(idx)
        .and_then(|m| Some(&m[3]))
        .ok_or_else(|| BotError::ImageParseError)?;

    Ok(image_url.to_string())
}
