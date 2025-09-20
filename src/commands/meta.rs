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
