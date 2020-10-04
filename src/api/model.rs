// use chrono::prelude::*;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use futures::future::TryFutureExt;
use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use url::Url;

use crate::error;

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct BragiInfoResponseBody {
    info: BragiInfo,
}

impl From<BragiInfo> for BragiInfoResponseBody {
    fn from(info: BragiInfo) -> Self {
        Self { info }
    }
}

#[derive(Debug, Deserialize)]
struct Env {
    name: String,
    url: String,
}

// This is used for POIs, to indicate if its a private or public source of POI.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, GraphQLEnum)]
#[serde(rename_all = "camelCase")]
pub enum PrivateStatus {
    Private,
    Public,
}

pub fn is_public(status: &PrivateStatus) -> bool {
    status == &PrivateStatus::Public
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, GraphQLEnum)]
#[serde(rename_all = "camelCase")]
pub enum ServerStatus {
    Available,
    NotAvailable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, GraphQLEnum)]
#[serde(rename_all = "camelCase")]
pub enum BragiStatus {
    Available,
    BragiNotAvailable,
    ElasticsearchNotAvailable,
}

#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
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
#[derive(Debug, Deserialize, Serialize, GraphQLObject)]
pub struct BragiStatusDetails {
    pub version: String,
    #[serde(rename = "es")]
    pub elasticsearch: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, GraphQLObject)]
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct ElasticsearhVersionDetails {
    pub number: String,
    #[serde(skip)]
    pub build_hash: String,
    #[serde(skip)]
    pub build_timestamp: String,
    #[serde(skip)]
    pub build_snapshot: String,
    #[serde(skip)]
    pub lucene_version: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, GraphQLObject)]
#[serde(rename_all = "camelCase")]
pub struct ElasticsearhInfoDetails {
    pub name: String,
    #[serde(skip)]
    pub cluster_name: String,
    #[serde(skip)]
    pub cluster_uuid: String,
    pub version: ElasticsearhVersionDetails,
    #[serde(skip)]
    pub tagline: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, GraphQLObject)]
pub struct ElasticsearchIndexInfo {
    pub label: String,
    pub place_type: String,
    pub coverage: String,
    #[serde(skip_serializing_if = "is_public")]
    pub private: PrivateStatus,
    pub date: DateTime<Utc>,
    pub count: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, GraphQLObject)]
pub struct ElasticsearchIndexInfoDetails {
    pub health: String,
    pub status: String,
    pub index: String,
    #[serde(skip)]
    pub prim: i32,
    #[serde(skip)]
    pub rep: i32,
    #[serde(rename = "docs.count")]
    pub count: String,
    #[serde(rename = "docs.deleted", skip)]
    pub deleted: String,
    #[serde(rename = "store.size", skip)]
    pub size: String,
    #[serde(rename = "pri.store.size", skip)]
    pub pri_size: String,
}

pub async fn status(url: &str) -> Result<BragiInfoResponseBody, error::Error> {
    let bragi_info = check_accessible(&url)
        .and_then(check_bragi_status)
        .and_then(check_elasticsearch_info)
        .and_then(check_elasticsearch_indices)
        .await?;

    Ok(BragiInfoResponseBody::from(bragi_info))
}

// Return a pair (environment, url)
// async fn get_url(env: &str) -> Result<(String, String), Error> {
//     let foo = std::fs::read_to_string("env.json").context(Config {
//         env: String::from("env.json"),
//     })?;
//
//     let foos: Vec<Env> = serde_json::from_str(&foo).context(DeserializeError)?;
//
//     let info: HashMap<String, String> = foos
//         .into_iter()
//         .map(|env| (env.name, env.url))
//         .collect::<HashMap<String, String>>();
//
//     info.get(env)
//         .ok_or(Error::Environment {
//             env: String::from(env),
//         })
//         .map(|s| (String::from(env), s.clone()))
// }

// Check that the url is accessible (should be done with some kind of 'ping')
// and return its arguments
async fn check_accessible(url: &str) -> Result<String, error::Error> {
    let status = reqwest::get(url)
        .await
        .context(error::NotAccessible { url: url.clone() })?
        .status();

    if status.is_client_error() || status.is_server_error() {
        Err(error::Error::MiscError {
            details: format!("Could not reach url {}", &url),
        })
    } else {
        Ok(String::from(url))
    }
}

async fn check_bragi_status(url: String) -> Result<BragiInfo, error::Error> {
    println!("checking {} bragi status", url);
    let status_url = format!("{}/status", url);
    let status: BragiStatusDetails = reqwest::get(&status_url)
        .await
        .context(error::NotAccessible { url: url.clone() })?
        .json()
        .await
        .context(error::NotReadable { url: url.clone() })?;

    // We brake the URL into its components, in order to get
    // the elastic search url, which may or may not include a port number
    // the name of the index, which is the first element in the path if it is present. If its not
    // present, we assign a sensible value by default. This could be improved.
    let elastic =
        Url::parse(&status.elasticsearch).context(error::ElasticsearchURLNotReadable {
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

    // FIXME Hardcode munin
    let prefix = String::from(elastic.path_segments().unwrap().next().unwrap_or("munin"));

    // We return a bragi info with empty elastic search indices... We delegate filling
    // this information to a later stage.
    Ok(BragiInfo {
        label: String::from("bragi"),
        url,
        version: status.version,
        status: BragiStatus::Available,
        elastic: Some(ElasticsearchInfo {
            label: String::from("elasticsearch"),
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

async fn check_elasticsearch_info(info: BragiInfo) -> Result<BragiInfo, error::Error> {
    let es_info = info.elastic.clone().ok_or(error::Error::MiscError {
        details: String::from("hello"),
    })?;
    let details: ElasticsearhInfoDetails = reqwest::get(&es_info.url)
        .await
        .context(error::NotAccessible {
            url: String::from(&es_info.url),
        })?
        .json()
        .await
        .context(error::NotAccessible {
            url: String::from(&es_info.url),
        })?;

    // TODO: We're not extracting much information now,
    // we need to get more...
    let es_update_info = ElasticsearchInfo {
        version: details.version.number,
        ..es_info
    };
    Ok(BragiInfo {
        elastic: Some(es_update_info),
        ..info
    })
}
//
// We retrieve all indices in json format, then use serde to deserialize into a data structure,
// and finally parse the label to extract the information.
async fn check_elasticsearch_indices(info: BragiInfo) -> Result<BragiInfo, error::Error> {
    let es_info = info.elastic.clone().ok_or(error::Error::MiscError {
        details: String::from("hello"),
    })?;
    let indices_url = format!("{}/_cat/indices?format=json", es_info.url);
    let indices: Vec<ElasticsearchIndexInfoDetails> = reqwest::get(&indices_url)
        .await
        .context(error::NotAccessible {
            url: String::from(&es_info.url),
        })?
        .json()
        .await
        .context(error::NotAccessible {
            url: String::from(&es_info.url),
        })?;

    let indices = indices
        .iter()
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
                coverage,
                private,
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
        .collect();

    let es_update_info = ElasticsearchInfo {
        status: ServerStatus::Available,
        indices,
        updated_at: Utc::now(),
        ..es_info
    };

    Ok(BragiInfo {
        elastic: Some(es_update_info),
        ..info
    })
}
