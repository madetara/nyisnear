use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Failed to reach yandex")]
    SearchImageError { source: reqwest::Error },

    #[error("Failed to parse URL from recieved html")]
    ImageParseError,

    #[error(transparent)]
    NetworkError(#[from] reqwest::Error),

    #[error("Race condition occured while trying ti update cache")]
    CacheRaceError,

    #[error("Cache hasn't been initialised")]
    CacheUninitialisedError,
}
