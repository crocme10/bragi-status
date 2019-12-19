use chrono::prelude::*;
//use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::{Host, ParseError, Url};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Available,
    NotAvailable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BragiStatus {
    Available,
    BragiNotAvailable,
    ElasticsearchNotAvailable,
}

#[derive(Debug)]
pub struct BragiInfo {
    pub label: String,
    pub url: String,
    pub version: String,
    pub status: BragiStatus,
    pub updated_at: DateTime<Utc>,
    pub elastic: Option<ElasticsearchInfo>,
}

// This struct is used to return the call to 'bragi/status'
// Its information will be inserted in the BragiStatus
#[derive(Debug, Deserialize)]
pub struct BragiStatusDetails {
    pub version: String,
    #[serde(rename = "es")]
    pub elasticsearch: String,
    pub status: String,
}

#[derive(Debug)]
pub struct ElasticsearchInfo {
    pub label: String,
    pub url: String,
    pub name: String,
    pub status: ServerStatus,
    pub version: String,
    pub indices: Vec<ElasticsearchIndexInfo>,
    pub index_prefix: String, // eg munin
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ElasticsearchIndexInfo {
    pub label: String,
    pub place_type: String,
    pub coverage: String,
    pub date: DateTime<Utc>,
    pub count: u32,
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
    let mut bragi = match info.get(&env) {
        Some(url) => BragiInfo {
            label: env,
            url: String::from(url),
            version: String::from(""),
            status: BragiStatus::Available,
            elastic: None,
            updated_at: Utc::now(),
        },
        None => BragiInfo {
            label: env,
            url: String::from(""),
            version: String::from(""),
            status: BragiStatus::BragiNotAvailable,
            elastic: None,
            updated_at: Utc::now(),
        },
    };
    if bragi.status == BragiStatus::BragiNotAvailable {
        println!("bragi: {:?}", bragi);
        std::process::exit(1);
    }
    let bragi_status = check_bragi_status(&bragi.url);
    if bragi_status.is_none() {
        println!("bragi: {:?}", bragi);
        std::process::exit(1);
    }
    update_bragi_info(&mut bragi, &bragi_status.unwrap());
    println!("bragi {:?}", bragi);
}

// fn check_backend_status(url: &str) -> ServerStatus {
//     match reqwest::blocking::get(url) {
//         Ok(resp) => {
//             if resp.status().is_success() {
//                 ServerStatus::Available
//             } else {
//                 ServerStatus::NotAvailable
//             }
//         }
//         Err(_) => ServerStatus::NotAvailable,
//     }
// }

fn check_bragi_status(url: &str) -> Option<BragiStatusDetails> {
    let status_url = format!("{}/status", url);
    match reqwest::blocking::get(&status_url) {
        Ok(resp) => match resp.json() {
            Ok(status) => Some(status),
            Err(err) => {
                println!("err deserialize {}", err);
                None
            }
        },
        Err(err) => {
            println!("cant get status {}", err);
            None
        }
    }
}

fn update_bragi_info(bragi: &mut BragiInfo, details: &BragiStatusDetails) {
    let elastic = Url::parse(&details.elasticsearch).unwrap();
    let elastic_url = match elastic.port() {
        None => format!("{}://{}", elastic.scheme(), elastic.host_str().unwrap()),
        Some(port) => format!(
            "{}://{}:{}",
            elastic.scheme(),
            elastic.host_str().unwrap(),
            port
        ),
    };
    let prefix = String::from(elastic.path_segments().unwrap().next().unwrap_or(""));

    bragi.elastic = Some(ElasticsearchInfo {
        label: bragi.label.clone(),
        url: elastic_url,
        name: String::from(""),
        status: ServerStatus::NotAvailable,
        version: details.version.clone(),
        indices: Vec::new(),
        index_prefix: prefix,
        updated_at: Utc::now(),
    });
}
