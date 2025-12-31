use tracing_subscriber::prelude::*;
use teloxide::prelude::*;
use tokio::sync::RwLock;
use anyhow::Result;
use dotenvy::dotenv;
use reqwest::Client;
use crate::{api::*, bot::*};
use std::{
    collections::HashMap,
    sync::Arc,
};

mod api;
mod bot;

type Users = Arc<RwLock<HashMap<i64, (f64, f64)>>>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let bot = Bot::from_env();
    let api_key = Arc::new(ApiKey::from_env());
    let client = Arc::new(Client::new());

    let users: Users = Arc::new(RwLock::new(HashMap::new()));

    let handle_start = Update::filter_message()
        .filter(|msg: Message| msg.text() == Some("/start"))
        .endpoint(start);
    
    let handle_location = Update::filter_message()
        .filter(|msg: Message| msg.location().is_some())
        .endpoint(handle_location);
    
    let handle_air = Update::filter_message()
        .filter(|msg: Message| {
            msg.text()
                .map(|t| t == "🌫 Качество воздуха")
                .unwrap_or(false)
        })
        .endpoint(air_quality);
    
    let handler = dptree::entry()
        .branch(handle_start)
        .branch(handle_location)
        .branch(handle_air);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![users, client, api_key])
        .build()
        .dispatch()
        .await;

    Ok(())
}
