extern crate image;
extern crate img_hash;

use crate::core::error::BotError;

use async_std::{fs, fs::DirEntry, prelude::*};
use image::DynamicImage;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use img_hash::{HashAlg, HasherConfig};
use rand::Rng;
use tokio::sync::RwLock;

const DIR: &str = "/data";

#[derive(Debug)]
struct CacheState {
    // seconds since UNIX Epoch
    updated_seconds: u64,
    cnt: u32,
    gen: u32,
    init: bool,
    hashes: HashSet<String>,
}

pub struct ImageCache {
    state: RwLock<CacheState>,
}

impl ImageCache {
    pub async fn new() -> Result<ImageCache> {
        let cache = ImageCache {
            state: RwLock::new(CacheState {
                updated_seconds: 0,
                cnt: 0,
                gen: 0,
                init: false,
                hashes: HashSet::new(),
            }),
        };

        cache.init_cache().await?;

        Ok(cache)
    }

    pub async fn get_random_image(&self) -> Result<Bytes> {
        let state = self.state.read().await;

        if state.cnt == 0 {
            return Err(anyhow!(BotError::CacheIsEmpty));
        }

        let idx = rand::thread_rng().gen_range(0, state.cnt);
        let path = get_path(idx);

        let data = fs::read(path).await?;

        Ok(Bytes::from(data))
    }

    pub async fn add_image(&self, img: Bytes) -> Result<()> {
        let mut state = self.state.write().await;

        tracing::info!("Start updating cache. Old generation: {:?}", state.gen);

        if !state.init {
            tracing::warn!("Attempted to update unintialised cache");
            return Err(anyhow!(BotError::CacheUninitialisedError));
        }

        let loaded_image =
            image::load_from_memory(img.as_ref()).context("Failed to load image from bytes")?;

        let hash = get_hash(loaded_image);

        if state.hashes.contains(&hash) {
            tracing::info!("Image already cached");
            return Ok(());
        }

        let path = get_path(state.cnt);
        fs::write(path, img).await?;

        state.hashes.insert(hash);
        state.updated_seconds = now_seconds();
        state.cnt += 1;
        state.gen += 1;

        tracing::info!("Cache updated. New generation: {:?}", state.gen);

        Ok(())
    }

    pub async fn is_stale(&self) -> bool {
        let state = self.state.read().await;

        !state.init || now_seconds() - state.updated_seconds >= 24 * 60 * 60 || state.cnt == 0
    }

    async fn init_cache(&self) -> Result<()> {
        tracing::info!("Initialising cache.");
        let mut state = self.state.write().await;

        if state.init {
            tracing::error!("Cache already initialised");
            return Err(anyhow!(BotError::CacheRaceError));
        }

        match fs::read_dir(DIR).await {
            Ok(_) => {}
            Err(err) => {
                tracing::info!(
                    "Error while looking for cache dir: {:?}. Trying to create",
                    err.to_string()
                );
                fs::create_dir(DIR).await?
            }
        }

        let mut dir = fs::read_dir(DIR).await?;
        let mut cnt = 0;
        let mut hashes: HashSet<String> = HashSet::new();

        while let Some(res) = dir.next().await {
            let entry: DirEntry = res?;

            let img = image::open(entry.path())?;
            let hash = get_hash(img);

            hashes.insert(hash);
            cnt += 1;
        }

        state.cnt = cnt;
        state.hashes = hashes;
        state.init = true;

        tracing::info!("Cache initialised: {:?}", state);

        Ok(())
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Tie went backwards")
        .as_secs()
}

fn get_hash(img: DynamicImage) -> String {
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher();

    hasher.hash_image(&img).to_base64()
}

fn get_path(idx: u32) -> PathBuf {
    Path::new(DIR).join(idx.to_string())
}
