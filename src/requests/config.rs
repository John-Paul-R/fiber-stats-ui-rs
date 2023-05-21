use fibermc_sdk::apis::configuration::Configuration;
use once_cell::sync::Lazy;

pub static REQUEST_CONFIG: Lazy<Configuration> = Lazy::new(|| Configuration {
    // base_path: "https://www.fibermc.com".to_owned(),
    base_path: "https://dev.fibermc.com".to_owned(),
    // base_path: "https://localhost:5001".to_owned(),
    user_agent: Some("OpenAPI-Generator/0.0.1/rust".to_owned()),
    client: reqwest::Client::new(),
    basic_auth: None,
    oauth_access_token: None,
    bearer_access_token: None,
    api_key: None,
});
