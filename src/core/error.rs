use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Failed to download chosen image")]
    GetImageError { source: reqwest::Error },

    #[error("Failed to reach yandex")]
    SearchImageError { source: reqwest::Error },

    #[error("Failed to parse URL from recieved html")]
    ImageParseError,

    #[error(transparent)]
    NetworkError(#[from] reqwest::Error),
}
