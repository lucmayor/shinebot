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
                                \n!bus - Gives bus times relative to provided WT stop grouping.",
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
        if let Ok(num) = input[0].parse::<i64>() {
            num
        } else {
            100
        }
    };

    let mut rng = rand::rng();
    let rand: f64 = rng.random::<f64>() * roll_bound as f64;
    let res: i64 = ((rand / 100_000.0).round() as i64) * 100_000;

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

    let len_check: i64 = (&disection).len() as i64;
    let combo_str: String = {
        if combo_res {
            if init_char == '1' {
                match len_check {
                    3 => {"dubs!!".to_owned()},
                    4 => {"trips!!!".to_owned()},
                    5 => {"quads!!!!".to_owned()},
                    _ => {"big number!!!!!".to_owned()}
                }
            } else {
                match len_check {
                    2 => {"dubs!!".to_owned()},
                    3 => {"trips!!!".to_owned()},
                    4 => {"quads!!!!".to_owned()},
                    _ => {"big number!!!!!".to_owned()}
                }
            }
        } else {
            "".to_owned()
        }
    };

    msg.channel_id.say(&ctx.http, format!("{} ... {}", res, combo_str)).await?;

    todo!()
}

// #[command]
// async fn choose(ctx: &Context, msg: &Message) -> CommandResult {
//     let input: Vec<String> = msg
//         .content
//         .split('|')
//         .map(|x| x.trim().to_string())
//         .collect();
//     let mut rng = rand::thread_rng();

//     let res = match input.choose(&mut rng) {
//         Some(x) => x,
//         None => "failed",
//     };

//     msg.channel_id.say(&ctx.http, format!("{:?}", res)).await?;

//     Ok(())
// }
