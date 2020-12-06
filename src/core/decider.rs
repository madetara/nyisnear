use chrono::{Datelike, Utc};
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;

pub async fn should_respond(msg: &str) -> bool {
    lazy_static! {
        static ref NY_REGEX: Regex = Regex::new(
            r"((\b|^|\W)[Нн][Гг](\b|$|\W))|((\b|^|\W)[Хх][Аа][Тт][Аа](\b|$|\W))|((\b|^|\W)[Нн]ов(ый|ым|ому) [Гг]од(ом|у)?(\b|$|\W))"
        )
        .unwrap();
    }

    let mut rng = rand::thread_rng();
    let now = Utc::now();

    now.month() == 12 && NY_REGEX.is_match(msg) && rng.gen_bool(0.5)
}
