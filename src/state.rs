use slog::{o, Logger};

use crate::error;
use crate::settings::Settings;

#[derive(Clone, Debug)]
pub struct State {
    pub logger: Logger,
    pub settings: Settings,
}

impl State {
    pub async fn new(settings: &Settings, logger: &Logger) -> Result<Self, error::Error> {
        let bragi_url = format!("http://{}:{}", settings.bragi.host, settings.bragi.port);
        let logger = logger.new(
            o!("host" => String::from(&settings.service.host), "port" => settings.service.port, "bragi" => bragi_url),
        );

        Ok(Self {
            logger,
            settings: settings.clone(),
        })
    }
}
