#![allow(deprecated)]
mod commands;

use std::env;
use std::collections::HashSet;
use std::sync::Arc;

use serenity::all::{Event, ResumedEvent, StandardFramework};
use serenity::{async_trait, client};
use serenity::framework::standard::Configuration;
use serenity::http::Http;
use serenity::gateway::ShardManager;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::all::standard::macros::group;
use tracing::{error, info};

use crate::commands::time::*;
use crate::commands::meta::*;
use crate::commands::owner::*;
use crate::prefix::TODO_COMMAND; // this may be shite

struct Handler;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _:Context, ready: Ready) {
        println!("{} connected!", ready.user.name);
    }

    async fn resume(&self, _:Context, _: ResumedEvent) {
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

    let token = env::var("DISCORD_KEY")
        .expect("Expected token in env.");
    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            (owners, info.id)
        },
        Err(msg) => panic!("App info error: {:?}", msg)
    };

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new()
                                    .owners(owners)
                                    .prefix("!"));

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
