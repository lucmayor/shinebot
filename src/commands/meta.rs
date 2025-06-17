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

// simple choose command
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
