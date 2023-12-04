use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use bytes::Bytes;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use tokio::sync::RwLock;

use crate::core::error::BotError;
use crate::core::imgcache::ImageCache;

const SEARCH_REQUEST: &str = "https://yandex.ru/images/search?text=%D1%85%D0%B0%D1%82%D0%B0 %D0%BD%D0%B0 %D0%BD%D0%B3 %D0%BC%D0%B5%D0%BC";

struct UpdateState {
    // UNIX timestamp
    attempt_time: u64,
}

pub struct ImageSource {
    cache: ImageCache,
    update_state: RwLock<UpdateState>,
}

impl ImageSource {
    pub async fn new() -> Result<ImageSource> {
        let cache = ImageCache::new().await?;

        Ok(ImageSource {
            cache,
            update_state: RwLock::new(UpdateState { attempt_time: 0 }),
        })
    }

    pub async fn get_image(&self) -> Result<Bytes> {
        self.get_from_cache().await
    }

    async fn get_from_cache(&self) -> Result<Bytes> {
        let last_update_attempt = {
            let state = self.update_state.read().await;
            state.attempt_time
        };

        if self.cache.is_stale().await && self.can_update().await {
            let mut state = self.update_state.write().await;

            if last_update_attempt == state.attempt_time {
                state.attempt_time = unix_now();
                self.update_cache().await?;
            }
        }

        self.cache.get_random_image().await
    }

    async fn update_cache(&self) -> Result<()> {
        lazy_static! {
            static ref IMG_REGEX: Regex =
                Regex::new(r"img_url=((http|https)%3A%2F%2F([\w_-]+(?:(?:\.[\w_-]+)+))([\w.,@?^=%&:/~+#-]*[\w@?^=%&/~+#-]))")
                    .unwrap();

            static ref REPLACE_REGEX: Regex =
                Regex::new(r"&(amp|quot)$")
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

        if found_images.is_empty() {
            if self.cache.get_items_count().await > 0 {
                tracing::warn!("Failed to get image URLs. Skipping cache update");
                return Ok(());
            }

            return Err(anyhow!(BotError::ImageParseError));
        }

        tracing::info!("Found {:?} images", found_images.len());

        let mut images_loaded = 0;

        for capture in found_images {
            tracing::info!("Processing capture {:?}", capture);
            let image_url = REPLACE_REGEX.replace_all(&capture[3], "");
            let image_url = url_escape::decode(&image_url);

            match self.load_and_save_image(&image_url).await {
                Err(err) => {
                    tracing::warn!(
                        "Failed to load image from {:?}. With error {:?}",
                        image_url,
                        err
                    );
                    continue;
                }
                Ok(()) => images_loaded += 1,
            };
        }

        tracing::info!("Loaded {:?} images", images_loaded);

        Ok(())
    }

    async fn load_and_save_image(&self, url: &str) -> Result<()> {
        tracing::info!("Downloading image");
        let image = reqwest::get(url).await?.bytes().await?;

        tracing::info!("Adding image to cache");
        self.cache.add_image(image).await?;

        Ok(())
    }

    async fn can_update(&self) -> bool {
        let update_state = self.update_state.read().await;
        unix_now() - update_state.attempt_time > 60 * 60
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
