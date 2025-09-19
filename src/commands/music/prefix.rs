#[allow(unreachable_code)]
use serenity::framework::standard::macros::command;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::framework::standard::CommandResult;

use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::DatabaseContainer;

struct LastfmResult {
    user: String,
    track: String,
    artist: String,
    album: String,
    now_playing: bool,
}

#[command]
#[aliases("nowplaying", "now")]
pub async fn np(ctx: &Context, msg: &Message) -> CommandResult {
    dotenv().ok();
    let api_key = &std::env::var("LASTFM_API").expect("lfm api key of doom");

    let data = ctx.data.read().await;
    let db = data.get::<DatabaseContainer>().expect("Fail finding DB");

    let user: i64 = msg.author.id.get() as i64;
    let query = sqlx::query!("SELECT * FROM lastfm WHERE id = ?;", user)
        .fetch_all(db)
        .await
        .unwrap();
    if query.len() == 0 {
        msg.channel_id
            .say(
                &ctx.http,
                "you gotta attach your account first white baby: `!setlfm [user]`",
            )
            .await?;
    } else {

        let lfm_user: String =
            <std::option::Option<std::string::String> as Clone>::clone(&query[0].lastfm).unwrap();
        let req_str: String = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", lfm_user, api_key);
        let body = reqwest::get(req_str).await?.text().await?;

        println!("finding track for user \"{}\" (lfm: {})", msg.author.name, &lfm_user);

        let v: Value = serde_json::from_str(&body)?;
        let music_val = &match v
            .get("recenttracks")
            .and_then(|a| a.get("track"))
            .and_then(|b| b.as_array())
        {
            Some(c) => c,
            None => {
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!("no tracks found for user `{}`", msg.author.name),
                    )
                    .await?;
                panic!("no track for user")
            }
        }[0];

        let music_res = LastfmResult {
            user: lfm_user,
            track: music_val.get("name").unwrap().to_string(),
            artist: music_val
                .get("artist")
                .and_then(|f| f.get("#text"))
                .unwrap()
                .to_string(),
            album: music_val
                .get("album")
                .and_then(|f| f.get("#text"))
                .unwrap()
                .to_string(),
            now_playing: {
                match music_val.get("@attr") {
                    Some(_) => true,
                    None => false,
                }
            },
        };

        match music_res.now_playing {
            true => {
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "`{}` is listening to: **{} - {}**",
                            music_res.user, music_res.artist, music_res.track
                        ),
                    )
                    .await?;
            }
            false => {
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "`{}` was last listening to: **{:} - {:}**",
                            music_res.user, music_res.artist, music_res.track
                        ),
                    )
                    .await?;
            }
        }
    }

    Ok(())
}

#[command]
#[aliases("setlastfm")]
pub async fn setlfm(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<DatabaseContainer>().expect("can't find db");
    let inp = msg.content.split(' ').collect::<Vec<&str>>();

    match (&inp).len() {
        1 => {
            msg.channel_id
                .say(&ctx.http, "usage: `!setlfm [username]`")
                .await?;
        }
        2 => {
            let uid = msg.author.id.get() as i64;
            let _ = sqlx::query!("INSERT INTO lastfm (id, lastfm) VALUES (?, ?)", uid, inp[1])
                .execute(db)
                .await;
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "set last.fm username for `{}` to `{}`",
                        msg.author.name, inp[1]
                    ),
                )
                .await?;
            println!(
                "set last.fm username for {} (uid: {}) to {}",
                msg.author.name, uid, inp[1]
            );
        }
        _ => {
            msg.channel_id
                .say(&ctx.http, "too many input vals! try again...")
                .await?;
        }
    }

    Ok(())
}
