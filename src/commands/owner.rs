use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::{DatabaseContainer, ShardManagerContainer};

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(mgr) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        mgr.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }

    Ok(())
}

#[command]
#[owners_only]
async fn last_used(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(db) = data.get::<DatabaseContainer>() {
        let quer = sqlx::query!("SELECT ts FROM stats ORDER BY entry_id DESC")
            .fetch_one(db)
            .await;

        match quer {
            Ok(res) => {
                let _ = msg.reply(ctx, format!("Last db cycle update: {}", res.ts)).await;
            },
            Err(e) => {
                let _ = msg.reply(ctx, format!("Error: {}", e)).await;
            }
        }
    } else {
        let _ = msg.reply(ctx, "Couldn't get to the DB").await;
    }

    Ok(())
}
