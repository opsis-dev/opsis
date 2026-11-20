use sqlx::SqlitePool;

pub const DEFAULT_LOCAL_ADDRESS: &str = "http://127.0.0.1:11434/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    Local,
    Anthropic,
    OpenAi,
    OpenRouter,
    Compatible,
}

impl Service {
    pub const ALL: [Self; 5] = [
        Self::Local,
        Self::Anthropic,
        Self::OpenAi,
        Self::OpenRouter,
        Self::Compatible,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::OpenRouter => "openrouter",
            Self::Compatible => "compatible",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|service| service.id() == id)
    }

    pub const fn needs_key(self) -> bool {
        !matches!(self, Self::Local)
    }

    pub const fn takes_address(self) -> bool {
        matches!(self, Self::Local | Self::Compatible)
    }

    pub fn key_name(self) -> String {
        format!("assistant:{}", self.id())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSettings {
    pub service: Service,
    pub model: String,
    pub address: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(
        "{0} isn't a model service. Use one of local, anthropic, openai, openrouter or compatible."
    )]
    UnknownService(String),
    #[error("A model name is needed.")]
    NoModel,
    #[error("{0} needs the address of an OpenAI-compatible server.")]
    NoAddress(&'static str),
}

impl ModelSettings {
    pub fn new(service: &str, model: &str, address: Option<&str>) -> Result<Self, SettingsError> {
        let service = Service::from_id(service.trim())
            .ok_or_else(|| SettingsError::UnknownService(service.to_owned()))?;
        let model = model.trim();
        if model.is_empty() {
            return Err(SettingsError::NoModel);
        }
        let address = address
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(str::to_owned);
        let address = match (service, address) {
            (Service::Local, None) => Some(DEFAULT_LOCAL_ADDRESS.to_owned()),
            (Service::Compatible, None) => return Err(SettingsError::NoAddress(service.id())),
            (service, address) if service.takes_address() => address,
            _ => None,
        };
        Ok(Self {
            service,
            model: model.to_owned(),
            address,
        })
    }
}

pub async fn read(pool: &SqlitePool) -> Result<Option<ModelSettings>, SettingsError> {
    let row = sqlx::query!("SELECT service, model, address FROM model_service WHERE id = 1")
        .fetch_optional(pool)
        .await?;
    row.map(|row| ModelSettings::new(&row.service, &row.model, row.address.as_deref()))
        .transpose()
}

pub async fn write(pool: &SqlitePool, settings: &ModelSettings) -> Result<(), SettingsError> {
    let service = settings.service.id();
    sqlx::query!(
        "INSERT INTO model_service (id, service, model, address) VALUES (1, ?, ?, ?)
         ON CONFLICT (id) DO UPDATE SET service = excluded.service, model = excluded.model, address = excluded.address",
        service,
        settings.model,
        settings.address
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn every_service_reads_back_from_its_id(index in 0..Service::ALL.len()) {
            let service = Service::ALL[index];
            prop_assert_eq!(Service::from_id(service.id()), Some(service));
            prop_assert!(service.key_name().starts_with("assistant:"));
        }

        #[test]
        fn only_servers_given_by_address_keep_one(index in 0..Service::ALL.len(), address in "https?://[a-z]{1,8}(:[0-9]{2,5})?") {
            let service = Service::ALL[index];
            let settings = ModelSettings::new(service.id(), "m", Some(&address)).unwrap();
            prop_assert_eq!(settings.address.is_some(), service.takes_address());
        }
    }

    #[test]
    fn a_local_server_defaults_to_ollama_and_a_compatible_one_needs_an_address() {
        let local = ModelSettings::new("local", "qwen3", None).unwrap();
        assert_eq!(local.address.as_deref(), Some(DEFAULT_LOCAL_ADDRESS));
        assert!(matches!(
            ModelSettings::new("compatible", "m", None),
            Err(SettingsError::NoAddress(_))
        ));
        assert!(matches!(
            ModelSettings::new("chatgpt", "m", None),
            Err(SettingsError::UnknownService(_))
        ));
        assert!(matches!(
            ModelSettings::new("openai", " ", None),
            Err(SettingsError::NoModel)
        ));
    }
}
