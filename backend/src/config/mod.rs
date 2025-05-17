use std::env;

pub struct Config{
    pub host : String,
    pub port : u16,

}

impl Config{
    pub fn from_env()->Self {
        dotenv::dotenv().ok();

        let host = env::var("HOST").expect("HOST is not set");
        let port = env::var("PORT")
            .expect("PORT is not set")
            .parse::<u16>()
            .expect("PORT must be a valid u16");

        Config {
            host,
            port
        }
    }
}