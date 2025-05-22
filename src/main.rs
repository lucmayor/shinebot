#![allow(deprecated)]
mod commands;

use std::collections::HashSet;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use serenity::all::standard::macros::group;
use serenity::all::{GuildId, ResumedEvent, StandardFramework};
use serenity::async_trait;
use serenity::framework::standard::Configuration;
use serenity::gateway::ShardManager;
use serenity::http::Http;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::info;

use sqlx::{Pool, Sqlite};

use crate::commands::bus::prefix::*;
use crate::commands::meta::*;
use crate::commands::owner::*;
use crate::commands::time::prefix::*; // this isn't how you're supposed to do it, fix l8r

struct Handler {
    loop_status: AtomicBool,
}

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

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built");

        let ctx = Arc::new(ctx);

        if !self.loop_status.load(Ordering::Relaxed) {

            // thread to handle reminders
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            });

            // thread to handle updating the personal file for reminders
            let ctx2 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });

            self.loop_status.swap(true, Ordering::Relaxed);
        }
    }
}

#[group]
#[commands(todo, bus, ping, quit)]
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
        .event_handler(Handler {
            loop_status: AtomicBool::new(false),
        })
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

// method to do the reminder part
// move this to the /time/ part
async fn check_reminders(ctx: Context) {
    // currently errors out -- fix later
    // essentially just query, check for before timestamps, send message + kill instance
    
    //let Ok(reminder_data) = sqlx::query!("SELECT user_id, task_desc, time_stamp FROM tasks WHERE time_stamp");
}