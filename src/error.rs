use juniper::{graphql_value, FieldError, IntoFieldError};
use snafu::{Backtrace, Snafu};
use std::io;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Misc Error: {}", details))]
    #[snafu(visibility(pub))]
    MiscError { details: String },

    #[snafu(display("Config Error: {} => {}", details, source))]
    #[snafu(visibility(pub))]
    ConfigError {
        details: String,
        source: config::ConfigError,
    },

    #[snafu(display("Environment Variable Error: {} => {}", details, source))]
    #[snafu(visibility(pub))]
    EnvVarError {
        details: String,
        source: std::env::VarError,
        backtrace: Backtrace,
    },

    #[snafu(display("IO Error: {}", source))]
    #[snafu(visibility(pub))]
    IOError {
        source: io::Error,
        details: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Reqwest Error: {} {}", details, source))]
    #[snafu(visibility(pub))]
    ReqwestError {
        source: reqwest::Error,
        details: String,
        backtrace: Backtrace,
    },

    #[snafu(display("URL Error: {} {}", details, source))]
    #[snafu(visibility(pub))]
    URLError {
        source: url::ParseError,
        details: String,
    },

    #[snafu(display("Tokio IO Error: {}: {}", details, source))]
    #[snafu(visibility(pub))]
    TokioIOError {
        source: tokio::io::Error,
        details: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Tokio Task Error {}: {}", details, source))]
    #[snafu(visibility(pub))]
    TokioJoinError {
        details: String,
        source: tokio::task::JoinError,
    },

    #[snafu(display("Serde Json Error: {} => {}", details, source))]
    #[snafu(visibility(pub))]
    SerdeJSONError {
        details: String,
        source: serde_json::error::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Parse Int Error: {} => {}", details, source))]
    #[snafu(visibility(pub))]
    ParseIntError {
        details: String,
        source: std::num::ParseIntError,
    },

    #[snafu(display("Could not access url {}", url))]
    #[snafu(visibility(pub))]
    NotAccessible { url: String, source: reqwest::Error },

    // FIXME Not sure how to specify the source type here,
    // it's a serde deserialization error, but it requires a lifetime...
    #[snafu(display("JSON Status not readable {}", url))]
    #[snafu(visibility(pub))]
    NotReadable { url: String, source: reqwest::Error },

    #[snafu(display("elasticsearch url not parsable {}", url))]
    #[snafu(visibility(pub))]
    ElasticsearchURLNotReadable {
        url: String,
        source: url::ParseError,
    },
}

impl IntoFieldError for Error {
    fn into_field_error(self) -> FieldError {
        match self {
            err @ Error::MiscError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("User Error", graphql_value!({ "internal_error": errmsg }))
            }
            err @ Error::ConfigError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Configuration Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }
            err @ Error::EnvVarError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Environment Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::IOError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("IO Error", graphql_value!({ "internal_error": errmsg }))
            }

            err @ Error::TokioIOError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Tokio IO Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::SerdeJSONError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("Serde Error", graphql_value!({ "internal_error": errmsg }))
            }

            err @ Error::ReqwestError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Reqwest Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::URLError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new("URL Error", graphql_value!({ "internal_error": errmsg }))
            }

            err @ Error::TokioJoinError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Tokio Join Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::ParseIntError { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Parse Int Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::NotAccessible { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Not Accessible Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::NotReadable { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Not Readable Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }

            err @ Error::ElasticsearchURLNotReadable { .. } => {
                let errmsg = format!("{}", err);
                FieldError::new(
                    "Elasticsearch URL Not Readable Error",
                    graphql_value!({ "internal_error": errmsg }),
                )
            }
        }
    }
}
