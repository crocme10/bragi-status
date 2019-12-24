use chrono::prelude::*;
//use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use url::{Host, ParseError, Url};
use snafu::{ResultExt, Snafu};
use url::Url;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not identify environment {}", env))]
    Environment { env: String },

    #[snafu(display("Could not access url {}", url))]
    NotAccessible { url: String, source: reqwest::Error },

    #[snafu(display("Could not access url {}", url))]
    StatusNotAccessible { url: String, source: reqwest::Error },

    // FIXME Not sure how to specify the source type here,
    // it's a serde deserialization error, but it requires a lifetime...
    #[snafu(display("JSON Status not readable {}", url))]
    StatusNotReadable { url: String, source: reqwest::Error },

    #[snafu(display("elasticsearch url not parsable {}", url))]
    ElasticsearchURLNotReadable {
        url: String,
        source: url::ParseError,
    },

    #[snafu(display("deserialize"))]
    DeserializeError { source: serde_json::error::Error },

    #[snafu(display("lack of imagination: {}", msg))]
    MiscError { msg: String },
}

// This is used for POIs, to indicate if its a private or public source of POI.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PrivateStatus {
    Private,
    Public,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Available,
    NotAvailable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
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
    let arg = std::env::args().skip(1).next();
    let env = arg.unwrap_or(String::from("dev")); // This is the requested environment
    if let Err(err) = run(&env) {
        eprintln!("Error: {}", err);
    }
}

fn run(env: &str) -> Result<(), Error> {
    get_url(env)
        .and_then(|(env, url)| check_accessible(env, url))
        .and_then(|(env, url)| check_bragi_status(env, url))
        .and_then(|bragi| update_elasticsearch_indices(bragi))
        .and_then(|bragi| {
            let b = serde_json::to_string(&bragi).context(DeserializeError)?;
            println!("{}", b);
            Ok(())
        })
}

// Return a pair (environment, url)
fn get_url(env: &str) -> Result<(String, String), Error> {
    let info: HashMap<String, String> = [
        // ("local", "http://localhost:400"),
        ("local", "http://lemonde.fr"),
        ("dev", "http://bragi-ws.ctp.dev.canaltp.fr"),
        ("internal", "http://bragi-ws.ctp.dev.canaltp.fr"),
        ("prod", "http://vippriv-bragi-ws.mutu.prod.canaltp.prod"),
    ]
    .into_iter()
    .map(|(k, v)| (String::from(k.to_owned()), String::from(v.to_owned())))
    .collect();

    info.get(env)
        .ok_or(Error::Environment {
            env: String::from(env),
        })
        .map(|s| (String::from(env), s.clone()))
}

// Check that the url is accessible (should be done with some kind of 'ping')
// and return its arguments
fn check_accessible(env: String, url: String) -> Result<(String, String), Error> {
    match reqwest::blocking::get(&url) {
        Ok(_) => Ok((env, url)),
        Err(err) => Err(Error::NotAccessible {
            url: url,
            source: err,
        }),
    }
}

fn check_bragi_status(env: String, url: String) -> Result<BragiInfo, Error> {
    let status_url = format!("{}/status", url);
    let resp =
        reqwest::blocking::get(&status_url).context(StatusNotAccessible { url: url.clone() })?;
    let status: BragiStatusDetails = resp
        .json()
        .context(StatusNotReadable { url: url.clone() })?;
    let elastic = Url::parse(&status.elasticsearch).context(ElasticsearchURLNotReadable {
        url: String::from(status.elasticsearch),
    })?;

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
    println!("prefix: {}", prefix);

    // We return a bragi info with empty elastic search indices... We delegate filling
    // this information to a later stage.
    Ok(BragiInfo {
        label: format!("bragi_{}", env),
        url: url,
        version: status.version,
        status: BragiStatus::Available,
        elastic: Some(ElasticsearchInfo {
            label: format!("elasticsearch_{}", env),
            url: elastic_url,
            name: String::from(""),
            status: ServerStatus::NotAvailable,
            version: String::from(""),
            indices: Vec::new(),
            index_prefix: prefix,
            updated_at: Utc::now(),
        }),
        updated_at: Utc::now(),
    })
}

// We retrieve all indices in json format, then use serde to deserialize into a data structure,
// and finally parse the label to extract the information.
fn update_elasticsearch_indices(info: BragiInfo) -> Result<BragiInfo, Error> {
    info.elastic
        .clone()
        .ok_or(Error::MiscError {
            msg: String::from("hello"),
        })
        .map(|es_info| {
            let indices_url = format!("{}/_cat/indices?format=json", es_info.url);
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
            let es_update_info = ElasticsearchInfo {
                status: status,
                indices: indices.unwrap_or(Vec::new()),
                updated_at: Utc::now(),
                ..es_info
            };
            BragiInfo {
                elastic: Some(es_update_info),
                ..info
            }
        })
}
