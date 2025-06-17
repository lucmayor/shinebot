#![allow(deprecated)]
mod commands;

use std::collections::HashSet;
use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use anyhow::Result;

use chrono::Local;
use serenity::all::standard::macros::group;
use serenity::all::{CreateMessage, GuildId, ResumedEvent, StandardFramework, UserId};
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

struct Record {
    taskid: i64,
    user_id: i64,
    task_desc: String,
    time_stamp: i64,
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

        println!("STATUS: {} connected!", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("STATUS: Cache built");

        let ctx = Arc::new(ctx);

        if !self.loop_status.load(Ordering::Relaxed) {
            // thread to handle reminders
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    println!(
                        "TASK: Checking reminders before timestamp {:?}",
                        Local::now()
                    );
                    let _ = check_reminders(&ctx1).await;
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            });

            // thread to handle updating the personal file for reminders
            // not implemented yet, not spawning this thread yet

            // let ctx2 = Arc::clone(&ctx);
            // tokio::spawn(async move {
            //     loop {
            //         tokio::time::sleep(Duration::from_secs(60)).await;
            //     }
            // });

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
        | GatewayIntents::GUILDS
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
// move this to the /time/ part if you can
async fn check_reminders(ctx: &Context) -> Result<()> {
    let data = ctx.data.read().await;
    let conn = data.get::<DatabaseContainer>().expect("DB not found");
    let mut failflag = false;

    let curr = Local::now().timestamp();
    let reminder_data = sqlx::query!(
        "SELECT taskid, user_id, task_desc, time_stamp 
        FROM tasks 
        WHERE time_stamp <= ?
        AND status = 'PENDING'
        AND type = 'in'",
        curr
    )
    .fetch_all(conn)
    .await?;

    let mut failed: Vec<Record> = Vec::new();

    for record in reminder_data {
        let exp_user = UserId::new(record.user_id as u64);
        let build = CreateMessage::new().content(format!(
            "<t:{:?}> : {:?} elapsed <t:{:?}:R>.",
            record.time_stamp, record.task_desc, record.time_stamp
        ));

        if let Err(what) = exp_user.direct_message(&ctx, build).await {
            dbg!(format!(
                "ERROR: Failed to send reminder to user {:?} for timestamp {:?} with message {:?}: {:?}",
                record.user_id, record.time_stamp, record.task_desc, what
            ));

            failflag = true;

            // maybe can add a from impl for Record
            // but would probably need to add to the actual sqlx query if anything
            failed.push(Record {
                taskid: record.taskid,
                user_id: record.user_id,
                task_desc: record.task_desc,
                time_stamp: record.time_stamp,
            });
            // can add error response here later, flesh out rest first
        }
    }

    // there is 100% a better way about this
    // this might be a very expensive manner of approaching this
    for fails in failed {
        let _ = sqlx::query!(
            "UPDATE tasks SET status = 'FAILED' WHERE taskid = ?",
            fails.taskid
        )
        .execute(conn)
        .await;
    }

    let _ = sqlx::query!(
        "UPDATE tasks 
        SET status = 'SENT' 
        WHERE time_stamp <= ? 
        AND status != 'FAILED'",
        curr
    )
    .execute(conn)
    .await;

    let _ = sqlx::query!("INSERT INTO stats (ts, fails) VALUES (?1, ?2)", curr, failflag)
    .execute(conn)
    .await;

    Ok(())
}
