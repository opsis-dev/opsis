use rig_core::DynModel;
use rig_core::operation::Completion;
use rig_core::providers::anthropic::Anthropic;
use rig_core::providers::openai::{OpenAI, OpenAIConfig};
use rig_core::providers::openrouter;

use crate::settings::{ModelSettings, Service};

const NO_KEY: &str = "none";

pub fn connect(settings: &ModelSettings, key: Option<&str>) -> DynModel<Completion> {
    let key = key.unwrap_or(NO_KEY).to_owned();
    let model = settings.model.clone();
    match settings.service {
        Service::Local | Service::Compatible => {
            let address = settings.address.clone().unwrap_or_default();
            OpenAIConfig::new(key)
                .with_base_url(address)
                .client()
                .chat(model)
                .erase()
        }
        Service::OpenAi => OpenAI::new(key).completion(model).erase(),
        Service::Anthropic => Anthropic::new(key).completion(model).erase(),
        Service::OpenRouter => openrouter::new(key).chat(model).erase(),
    }
}
