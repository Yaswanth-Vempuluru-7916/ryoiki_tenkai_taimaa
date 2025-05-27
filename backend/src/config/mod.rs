use std::env;

pub struct Config{
    pub host : String,
    pub port : u16,
    pub database_url : String

}

impl Config{
    pub fn from_env()->Self {
        dotenv::dotenv().expect("Failed to load .env file");

        let host = env::var("HOST").expect("HOST is not set");
        let port = env::var("PORT")
            .expect("PORT is not set")
            .parse::<u16>()
            .expect("PORT must be a valid u16");
        let database_url = env::var("DATABASE_URL").expect("DB_URL not found");

        Config {
            host,
            port,
            database_url
        }
    }
}