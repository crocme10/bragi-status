use chrono::prelude::*;
//use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use url::{Host, ParseError, Url};
use url::Url;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrivateStatus {
    Private,
    Public,
}

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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct ElasticsearchIndexInfo {
    pub label: String,
    pub place_type: String,
    pub coverage: String,
    #[serde(skip_serializing_if = "is_public")]
    pub private: PrivateStatus,
    pub date: DateTime<Utc>,
    pub count: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ElasticsearchIndexInfoDetails {
    pub health: String,
    pub status: String,
    pub index: String,
    #[serde(skip)]
    pub prim: u32,
    #[serde(skip)]
    pub rep: u32,
    #[serde(rename = "docs.count")]
    pub count: String,
    #[serde(rename = "docs.deleted", skip)]
    pub deleted: String,
    #[serde(rename = "store.size", skip)]
    pub size: String,
    #[serde(rename = "pri.store.size", skip)]
    pub pri_size: String,
}

fn is_public(status: &PrivateStatus) -> bool {
    status == &PrivateStatus::Public
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
    bragi.elastic = bragi.elastic.map(update_elasticsearch_indices);
    let b = serde_json::to_string(&bragi).unwrap();
    println!("{}", b);
}

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

// Extract the information from BragiStatusDetails, and store it in the mutable BragiInfo
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

// We retrieve all indices in json format, then use serde to deserialize into a data structure,
// and finally parse the label to extract the information.
fn update_elasticsearch_indices(info: ElasticsearchInfo) -> ElasticsearchInfo {
    let indices_url = format!("{}/_cat/indices?format=json", info.url);
    let indices: Option<Vec<ElasticsearchIndexInfo>> = reqwest::blocking::get(&indices_url)
        .ok()
        .and_then(|resp| resp.json().ok())
        .map(|is: Vec<ElasticsearchIndexInfoDetails>| {
            is.iter()
                .map(|i| {
                    let zs: Vec<&str> = i.index.split('_').collect();
                    let (private, coverage) = if zs[2].starts_with("priv.") {
                        (PrivateStatus::Private, zs[2].chars().skip(5).collect())
                    } else {
                        (PrivateStatus::Public, zs[2].to_string())
                    };
                    ElasticsearchIndexInfo {
                        label: i.index.clone(),
                        place_type: zs[1].to_string(),
                        coverage: coverage,
                        private: private,
                        date: DateTime::<Utc>::from_utc(
                            NaiveDateTime::new(
                                NaiveDate::parse_from_str(zs[3], "%Y%m%d")
                                    .unwrap_or(NaiveDate::from_ymd(1970, 1, 1)),
                                NaiveTime::parse_from_str(zs[4], "%H%M%S")
                                    .unwrap_or(NaiveTime::from_hms(0, 1, 1)),
                            ),
                            Utc,
                        ),
                        count: i.count.parse().unwrap_or(0),
                        updated_at: Utc::now(),
                    }
                })
                .collect()
        });
    let status = if indices.is_some() {
        ServerStatus::Available
    } else {
        ServerStatus::NotAvailable
    };
    ElasticsearchInfo {
        label: info.label,
        url: info.url,
        name: info.name,
        status: status,
        version: info.version,
        indices: indices.unwrap_or(Vec::new()),
        index_prefix: info.index_prefix,
        updated_at: Utc::now(),
    }
}
