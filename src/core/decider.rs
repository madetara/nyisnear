use chrono::{Datelike, Local};
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;
use std::cmp;
use std::sync::atomic::{AtomicU8, Ordering};

pub async fn should_respond(msg: &str) -> bool {
    lazy_static! {
        static ref NY_REGEX: Regex = Regex::new(
            r"((\b|^|\W)[Нн][Гг](\b|$|\W))|((\b|^|\W)[Хх][Аа][Тт][Аа](\b|$|\W))|((\b|^|\W)[Нн]ов(ый|ым|ому) [Гг]од(ом|у)?(\b|$|\W))"
        )
        .unwrap();

        static ref COUNTER: AtomicU8 = AtomicU8::new(0);
    }

    let mut rng = rand::thread_rng();
    let now = Local::now();

    if now.month() != 12 || !NY_REGEX.is_match(msg) {
        tracing::info!("Message didn't pass preconditions");
        return false;
    }

    let probability = cmp::max((COUNTER.fetch_add(1, Ordering::SeqCst) * 10) % 100, 70) as f64;

    tracing::info!("Deciding with probability: {:.20}", probability);

    rng.gen_bool(probability / 100.0)
}
