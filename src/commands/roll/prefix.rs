use std::mem::discriminant;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use anyhow::Result;
use rand::prelude::*;
use sqlx::{Database, Sqlite};

use crate::DatabaseContainer;

#[derive(PartialEq)]
enum Special {
    None,
    Dubs,
    Trips,
    Quads,
    Above,
}

struct UserStats {
    id: i64,
    rolls: i64,
    dubs_counter: i64,
    trips_counter: i64,
    quads_counter: i64,
    above_counter: i64,
    elo: f64,
}

#[command]
async fn roll(ctx: &Context, msg: &Message) -> CommandResult {
    let input: Vec<String> = msg
        .content
        .split(' ')
        .map(|x| x.trim().to_string())
        .collect();

    let roll_bound = {
        if input.len() > 1 {
            if let Ok(num) = input[1].parse::<i64>() {
                num
            } else {
                100
            }
        } else {
            100
        }
    };

    let rand: f64 = {
        let mut rng = rand::rng();
        rng.random::<f64>() * roll_bound as f64
    };
    let res: i64 = rand.round() as i64;

    // dubs check, trips check, etc...
    let disection: Vec<char> = res.to_string().chars().collect();
    let init_char: char = disection[0];
    let combo_res: bool = {
        let mut matches: bool = true;
        if init_char == '1' && disection[1] != '1' {
            for i in &disection {
                if *i != '0' {
                    matches = false;
                }
            }
            matches
        } else {
            for i in &disection {
                if *i != init_char {
                    matches = false;
                }
            }
            matches
        }
    };

    // can be shortened do this whenever
    let len_check: i64 = (&disection).len() as i64;
    let combo_str: Special = {
        if len_check == 1 {
            Special::None
        } else {
            if combo_res {
                if init_char == '1' && disection[1] != '1' {
                    match len_check {
                        3 => Special::Dubs,
                        4 => Special::Trips,
                        5 => Special::Quads,
                        _ => Special::Above,
                    }
                } else {
                    match len_check {
                        2 => Special::Dubs,
                        3 => Special::Trips,
                        4 => Special::Quads,
                        _ => Special::Above,
                    }
                }
            } else {
                Special::None
            }
        }
    };

    msg.channel_id
        .say(
            &ctx.http,
            format!("{} ... {}", res, translate_special(combo_str)),
        )
        .await?;

    Ok(())
}

fn translate_special(sp: Special) -> String {
    match sp {
        Special::None => "".to_string(),
        Special::Dubs => "dubs!!".to_string(),
        Special::Trips => "trips!!!".to_string(),
        Special::Quads => "quads!!!!".to_string(),
        Special::Above => "big number matching !!!!!".to_string(),
    }
}

async fn special_elo(ctx: &Context, msg: &Message, sp: Special) -> Result<()> {
    let uid: i64 = msg.author.id.get() as i64;
    let data = ctx.data.read().await;
    let db = data.get::<DatabaseContainer>().expect("caent find db");

    let u = sqlx::query!("SELECT * FROM rolls WHERE id = ?", uid)
        .fetch_optional(db)
        .await
        .unwrap();
    let user_data = match u {
        Some(user) => {
            UserStats {
                id: uid,
                rolls: user.rolls.unwrap(),
                dubs_counter: user.dubs_counter.unwrap(),
                trips_counter: user.trips_counter.unwrap(),
                quads_counter: user.quads_counter.unwrap(),
                above_counter: user.above_counter.unwrap(),
                elo: user.elo.unwrap()
            }
        },
        None => {
            let _ = sqlx::query!("INSERT INTO rolls (id, rolls) VALUES (?, ?)", uid, 1)
                .execute(db)
                .await;
            
            UserStats {
                id: uid,
                rolls: 1,
                dubs_counter: 0,
                trips_counter: 0,
                quads_counter: 0,
                above_counter: 0,
                elo: 1600.0
            }
        }
    };

    let average = match sqlx::query!("SELECT AVG(elo) as avg FROM rolls WHERE NOT id = ?", uid)
        .fetch_optional(db)
        .await
        .unwrap() {
            Some(s) => s.avg.unwrap(),
            None => 1600.0
        };

    if sp == Special::None {
        // loss
    } else {
        // win
    }

    Ok(())
}

async fn chess_eq(roller_elo: f64, competitor_elo: f64, wins: i64, losses: i64) -> f64 {
    roller_elo + (16 * (wins - losses)) as f64 + (0.04 * (roller_elo - competitor_elo))
}
