use chrono::prelude::*;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendStatus {
    Available,
    NotAvailable,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Bragi,
    Elasticsearch,
}

#[derive(Debug)]
pub struct Backend {
    pub label: String,
    pub url: String,
    pub kind: BackendKind,
    pub status: BackendStatus,
    pub updated_at: DateTime<Utc>,
}

fn main() {
    let info: HashMap<String, String> = [
        ("local", "http://localhost:4000"),
        ("dev", "http://bragi-ws.ctp.dev.canaltp.fr"),
        ("internal", "http://bragi-ws.ctp.dev.canaltp.fr"),
        ("prod", "http://vippriv-bragi-ws.mutu.prod.canaltp.prod"),
    ]
    .into_iter()
    .map(|(k, v)| (String::from(k.to_owned()), String::from(v.to_owned())))
    .collect();

    let arg = std::env::args().skip(1).next();
    let dev = String::from("dev");
    let env = arg.unwrap_or(dev);
    let bragi = match info.get(&env) {
        Some(url) => Backend {
            label: env,
            url: String::from(url),
            kind: BackendKind::Bragi,
            status: check_backend_status(&url),
            updated_at: Utc::now(),
        },
        None => Backend {
            label: env,
            url: String::from(""),
            kind: BackendKind::Bragi,
            status: BackendStatus::NotAvailable,
            updated_at: Utc::now(),
        },
    };
    println!("bragi: {:?}", bragi);
}

fn check_backend_status(url: &str) -> BackendStatus {
    match reqwest::blocking::get(url) {
        Ok(resp) => {
            if resp.status().is_success() {
                BackendStatus::Available
            } else {
                BackendStatus::NotAvailable
            }
        }
        Err(_) => BackendStatus::NotAvailable,
    }
}
