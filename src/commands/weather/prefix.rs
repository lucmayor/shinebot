#[allow(unreachable_code)]
use serenity::{
    all::{CreateEmbed, CreateMessage},
    framework::standard::{macros::command, Args, CommandResult},
    model::{channel::Message, Colour, Timestamp as TimestampSer},
    prelude::*,
};
use anyhow::Result;
use tokio::task;

#[command]
#[aliases("weather", "w")]
pub async fn weather(ctx: &Context, msg: &Message) -> CommandResult {
    let client = reqwest::Client::new();

    

    Ok(())
}