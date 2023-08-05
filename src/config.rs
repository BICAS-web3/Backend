use std::env;

use lazy_static::lazy_static;
use serde::Deserialize;

// Env variables
lazy_static! {
    // datatabse config
    pub static ref DB_USERNAME: String = env::var("DB_USERNAME").unwrap();
    pub static ref DB_PASSWORD: String = env::var("DB_PASSWORD").unwrap();
    pub static ref DB_HOST: String = env::var("DB_HOST").unwrap();
    pub static ref DB_PORT: u16 = env::var("DB_PORT").unwrap().parse().unwrap();
    pub static ref DB_NAME: String = env::var("DB_NAME").unwrap();

    // server config
    pub static ref SERVER_HOST: String = env::var("SERVER_HOST").unwrap();
    pub static ref SERVER_PORT: String = env::var("SERVER_PORT").unwrap();

    // other params
    pub static ref PAGE_SIZE: i64 = env::var("PAGE_SIZE").unwrap().parse().unwrap();
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
