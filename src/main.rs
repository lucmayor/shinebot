#![allow(deprecated)]
mod commands;

use std::collections::HashSet;
use std::env;
use std::sync::Arc;

use serenity::all::standard::macros::group;
use serenity::all::{ResumedEvent, StandardFramework};
use serenity::framework::standard::Configuration;
use serenity::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::async_trait;
use tracing::info;

use sqlx::{Pool, Sqlite};

use crate::commands::meta::*;
use crate::commands::owner::*;
use crate::commands::time::prefix::*; // this isn't how you're supposed to do it, fix l8r

struct Handler;

pub struct ShardManagerContainer;
pub struct DatabaseContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

impl TypeMapKey for DatabaseContainer {
    type Value = Pool<Sqlite>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let data = ctx.data.read().await;
        let _db = data.get::<DatabaseContainer>().expect("Database not found");

        println!("{} connected!", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(todo, ping, quit)]
struct General;

// have to specify flavor for whatever reason
#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().expect("Failed loading environment!");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected token in env.");
    let http = Http::new(&token);

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(3)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to db");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't do db migration");

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            (owners, info.id)
        }
        Err(msg) => panic!("App info error: {:?}", msg),
    };

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().owners(owners).prefix("!"));

    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES;

    // might have to undo that addressing
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error in client creation.");

    {
        let mut data = client.data.write().await;
        data.insert::<DatabaseContainer>(database.clone());
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let _shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Bad register ctrl+c");
    });

    if let Err(err_msg) = client.start().await {
        println!("Client error: {err_msg:?}");
    }
}
