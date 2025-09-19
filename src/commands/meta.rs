use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use rand::prelude::*;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "pong").await?;
    Ok(())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(
            &ctx.http,
            "current commands: 
                                \n!todo - Sets reminder in database to do task.
                                \n!bus - Gives bus times relative to provided WT stop grouping.
                                \n!choose - Chooses between options.
                                \n!roll - Roll a number.
                                \n!np - Show listening status (sourced from last.fm).",
        )
        .await?;
    Ok(())
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
        if init_char == '1' {
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
    let combo_str: String = {
        if len_check == 1 {
            "".to_owned()
        } else {
            if combo_res {
                if init_char == '1' {
                    match len_check {
                        3 => "dubs!!".to_owned(),
                        4 => "trips!!!".to_owned(),
                        5 => "quads!!!!".to_owned(),
                        _ => "big number!!!!!".to_owned(),
                    }
                } else {
                    match len_check {
                        2 => "dubs!!".to_owned(),
                        3 => "trips!!!".to_owned(),
                        4 => "quads!!!!".to_owned(),
                        _ => "big number!!!!!".to_owned(),
                    }
                }
            } else {
                "".to_owned()
            }
        }
    };

    msg.channel_id
        .say(&ctx.http, format!("{} ... {}", res, combo_str))
        .await?;

    Ok(())
}

#[command]
async fn choose(ctx: &Context, msg: &Message) -> CommandResult {
    let mut input: Vec<String> = msg
        .content
        .split('|')
        .map(|x| x.trim().to_string())
        .collect();

    // hacky, but trims leading cmd string
    input[0] = input[0].split(' ').collect::<Vec<&str>>()[1]
        .trim()
        .to_string();

    let res = match {
        let mut rng = rand::thread_rng();
        input.choose(&mut rng)
    } {
        Some(x) => x,
        None => "failed",
    };

    msg.channel_id.say(&ctx.http, format!("{:?}", res)).await?;

    Ok(())
}
