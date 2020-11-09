use clap::ArgMatches;
use config::{Config, Environment, File};
use serde::Deserialize;
use snafu::ResultExt;
use std::env;
use std::path::PathBuf;

use super::error;

#[derive(Debug, Clone, Deserialize)]
pub struct Bragi {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub testing: bool,
    pub mode: String,
    pub service: Service,
    pub bragi: Bragi,
}

// TODO Parameterize the config directory

impl Settings {
    pub fn new<'a, T: Into<Option<&'a ArgMatches<'a>>>>(matches: T) -> Result<Self, error::Error> {
        let matches = matches.into().ok_or_else(|| error::Error::MiscError {
            details: String::from("Could not read CLI"),
        })?;

        let mut dir = PathBuf::from(matches.value_of("config").unwrap_or_else(|| "config"));

        let mut config = Config::new();

        // Start off by merging in the "default" configuration file
        dir.push("default");
        config
            .merge(File::with_name(&dir.to_str().expect("filename")))
            .context(error::ConfigError {
                details: format!(
                    "Could not merge default configuration from '{}'",
                    dir.display()
                ),
            })?;
        dir.pop();

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let mode = env::var("SETTINGS").unwrap_or_else(|_| String::from("development"));
        dir.push(&mode);
        config
            .merge(File::with_name(&dir.to_str().expect("filename")).required(true))
            .context(error::ConfigError {
                details: format!(
                    "Could not merge '{}' configuration from '{}'",
                    mode,
                    dir.display()
                ),
            })?;
        dir.pop();

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        dir.push("local");
        config
            .merge(File::with_name(&dir.to_str().expect("filename")).required(false))
            .context(error::ConfigError {
                details: format!(
                    "Could not merge 'local' configuration from '{}'",
                    dir.display(),
                ),
            })?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        config
            .merge(Environment::with_prefix("app"))
            .context(error::ConfigError {
                details: String::from("Could not merge configuration from environment variables"),
            })?;

        // Finally we override values with what has been given at the command line
        if let Some(addr) = matches.value_of("address") {
            config
                .set("service.host", addr)
                .context(error::ConfigError {
                    details: String::from("Could not set service host from CLI argument"),
                })?;
        }

        if let Some(port) = matches.value_of("port") {
            let _port = port.parse::<u16>().map_err(|err| error::Error::MiscError {
                details: format!("Could not parse into a valid port number ({})", err),
            })?;
            config
                .set("service.port", port)
                .context(error::ConfigError {
                    details: String::from("Could not set service port from CLI argument"),
                })?;
        }

        // You can deserialize (and thus freeze) the entire configuration as
        config.try_into().context(error::ConfigError {
            details: String::from("Could not generate settings from configuration"),
        })
    }
}
