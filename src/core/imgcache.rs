extern crate image;
extern crate image_hasher;

use anyhow::{anyhow, Context, Result};
use async_std::{
    fs,
    fs::DirEntry,
    path::{Path, PathBuf},
    prelude::*,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use image_hasher::{HashAlg, HasherConfig, Image};
use rand::Rng;
use tokio::sync::RwLock;

use std::{
    cmp::Ordering,
    collections::HashSet,
    io::Cursor,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::core::error::BotError;

const DIR: &str = "/data";
const LAST_UPDATE_STORE: &str = "last_update";

#[derive(Debug)]
struct CacheState {
    // seconds since UNIX Epoch
    updated_seconds: u64,
    cnt: u32,
    generation: u32,
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
                generation: 0,
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

        let idx = rand::rng().random_range(0..state.cnt);
        let path = get_path_to_image(idx);

        let data = fs::read(path).await?;

        Ok(Bytes::from(data))
    }

    pub async fn add_image(&self, img: Bytes) -> Result<()> {
        let mut state = self.state.write().await;

        tracing::info!(
            "Start updating cache. Old generation: {:?}",
            state.generation
        );

        if !state.init {
            tracing::warn!("Attempted to update unintialised cache");
            return Err(anyhow!(BotError::CacheUninitialisedError));
        }

        let loaded_image =
            image::load_from_memory(img.as_ref()).context("Failed to load image from bytes")?;

        let hash = get_hash(&loaded_image);

        if state.hashes.contains(&hash) {
            tracing::info!("Image already cached");
            return Ok(());
        }

        let path = get_path_to_image(state.cnt);
        fs::write(path, img).await?;

        let update_time = now_seconds();

        write_last_update_unsafe(update_time).await?;

        state.hashes.insert(hash);
        state.updated_seconds = update_time;
        state.cnt += 1;
        state.generation += 1;

        tracing::info!("Cache updated. New generation: {:?}", state.generation);

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
                fs::create_dir(DIR).await?;
            }
        }

        let mut dir = fs::read_dir(DIR).await?;
        let mut cnt = 0;
        let mut hashes: HashSet<String> = HashSet::new();

        while let Some(res) = dir.next().await {
            let entry: DirEntry = res?;

            // TODO: temporary solution. move to state and images to separate folders
            if entry.path().cmp(&get_path_to_update_time()) == Ordering::Equal {
                continue;
            }

            let raw_image = fs::read(entry.path()).await?;
            let img = image::load_from_memory(raw_image.as_slice())?;
            let hash = get_hash(&img);

            hashes.insert(hash);
            cnt += 1;
        }

        let last_update = match read_last_update_unsafe().await {
            Ok(last_update) => last_update,
            Err(err) => {
                tracing::warn!("Failed to read update time. Error: {:?}", err);
                let result = if cnt > 0 {
                    tracing::info!("Defaulting to current time.");
                    now_seconds()
                } else {
                    tracing::info!("Defaulting to zero.");
                    0
                };
                write_last_update_unsafe(result).await?;
                result
            }
        };

        state.updated_seconds = last_update;
        state.cnt = cnt;
        state.hashes = hashes;
        state.init = true;

        tracing::info!("Cache initialised: {:?}", state);

        Ok(())
    }

    pub async fn get_items_count(&self) -> u32 {
        let state = self.state.read().await;
        state.cnt
    }
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn get_hash<T: Image>(img: &T) -> String {
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher();

    hasher.hash_image(img).to_base64()
}

async fn read_last_update_unsafe() -> Result<u64> {
    let path = get_path_to_update_time();
    let raw = fs::read(path).await?;
    let mut rdr = Cursor::new(raw);

    Ok(rdr.read_u64::<BigEndian>()?)
}

async fn write_last_update_unsafe(last_update: u64) -> Result<()> {
    let path = get_path_to_update_time();
    let mut raw = vec![];

    raw.write_u64::<BigEndian>(last_update)?;
    fs::write(path, raw).await?;

    Ok(())
}

fn get_path_to_image(idx: u32) -> PathBuf {
    Path::new(DIR).join(idx.to_string())
}

fn get_path_to_update_time() -> PathBuf {
    Path::new(DIR).join(LAST_UPDATE_STORE)
}
