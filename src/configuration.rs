use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::mysql::MySqlConnectOptions;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Config {
    pub application: Application,
    pub database: Database,
    pub jwt: Jwt,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Application {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub allow_registration: bool,
    pub run_migration: bool,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Jwt {
    pub secret: Secret<String>,
    pub iss: Secret<String>,
    pub aud: Secret<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Database {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
}

impl Database {
    pub fn without_db(&self) -> MySqlConnectOptions {
        MySqlConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
    }
    pub fn with_db(&self) -> MySqlConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

impl Config {
    pub fn new() -> Result<Self, figment::Error> {
        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");
        let config_directory = base_path.join("configuration");

        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");

        let environment_filename = format!("{}.yaml", environment.as_str());

        Figment::new()
            .merge(Yaml::file(config_directory.join("base.yaml")))
            .merge(Yaml::file(config_directory.join(environment_filename)))
            .merge(Env::raw().split("__"))
            .extract()
    }
}
