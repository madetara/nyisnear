use anyhow::{anyhow, Result};
use bytes::Bytes;
use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::core::error::BotError;
use crate::core::imgcache::ImageCache;

const SEARCH_REQUEST: &str = "https://yandex.ru/images/search?text=%D1%85%D0%B0%D1%82%D0%B0 %D0%BD%D0%B0 %D0%BD%D0%B3 %D0%BC%D0%B5%D0%BC";

pub struct ImageSource {
    cache: ImageCache,
}

impl ImageSource {
    pub async fn new() -> Result<ImageSource> {
        let cache = ImageCache::new().await?;

        Ok(ImageSource { cache })
    }

    pub async fn get_image(&self) -> Result<Bytes> {
        Ok(self.get_from_cache().await?)
    }

    async fn get_from_cache(&self) -> Result<Bytes> {
        if self.cache.is_stale().await {
            self.update_cache().await?;
        }

        Ok(self.cache.get_random_image().await?)
    }

    async fn update_cache(&self) -> Result<()> {
        lazy_static! {
            static ref IMG_REGEX: Regex =
                Regex::new(r#""w":(\d+),"h":(\d+),"origin":\{[^\}]*(https?://[^"]*)[^<]*"#)
                    .unwrap();
        }

        tracing::info!("Updating cache");

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
            return Err(anyhow!(BotError::ImageParseError));
        }

        tracing::info!("Found {:?} images", found_images.len());

        for capture in found_images {
            let image_url = &capture[3];
            let image = reqwest::get(image_url).await?.bytes().await?;

            tracing::info!("Adding image to cache");
            self.cache.add_image(image).await?;
        }

        Ok(())
    }
}
