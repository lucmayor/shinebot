#[allow(unreachable_code)]

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::model::channel::Message;

use chrono::{Utc, NaiveDate, Datelike, Weekday};

use phf::phf_map;

static MONTHS: phf::Map<&'static str, i32> = phf_map! {
    "jan" => 1, "feb" => 2, "mar" => 3,
    "apr" => 4, "may" => 5, "jun" => 6,
    "jul" => 7, "aug" => 8, "sept" => 9,
    "oct" => 10, "nov" => 11, "dec" => 12,

    "january" => 1, "february" => 2, "march" => 3,
    "april" => 4, "june" => 6, "july" => 7,
    "august" => 8, "september" => 9, "october" => 10,
    "november" => 11, "december" => 12
};

#[command]
pub async fn todo(ctx: &Context, msg: &Message) -> CommandResult {
    // !do [descriptor] by [timestring]
    // !do [descriptor] in [timestring]
    if msg.content.starts_with("!do") {
        // fill w/ logic
        let user_string = &msg.content;

        // note, replace all this with regex: (.+?)\s+(by|in)\s+(.+)

        // strings past user_string[0], iterate until user_string[n] = "by", rest of string arr for dating
        // for dating, switch on date types (minutes / hours / days / months / years)
        
        // by case -> (specific date) / ('next' 'weekdate') .. we can have reminder time be stock for any given user / we have not 
        // in case -> (timeslot) (measurement)

        let mut casing: Option<&str> = None;

        let string_tuple: (String, String) = match user_string {
            _ if user_string.contains("by") => user_string.find(" by ")
                                                .map(|pos| {
                                                    let(before, after) = user_string.split_at(pos);
                                                    let date_part = &after[4..];
                                                    casing = Some("by");
                                                    (before.trim().to_string(), date_part.trim().to_string())
                                                }),
            _ if user_string.contains("in") => user_string.find(" in ")
                                                .map(|pos| {
                                                    let(before, after) = user_string.split_at(pos);
                                                    let date_part = &after[4..];
                                                    casing = Some("in");
                                                    (before.trim().to_string(), date_part.trim().to_string())
                                                }),
            _ => None
        }.unwrap();

        let date_fix: Option<i64> = match &string_tuple.1.chars().filter(|c| *c == ' ').count() {
            0 => Some(1), // temp
            1 => match &string_tuple.1 {
                // the [5th], [6th], [7th] case
                _ if (&string_tuple.1).contains("the") => {
                    let substr = &string_tuple.1[..&user_string.len()-2]; // 32nd
                    let date: u32 = substr.parse::<u32>().unwrap();

                    let curr_date = Utc::now().date_naive();
                    let new_date: Option<NaiveDate>;

                    match curr_date.day() {
                        _ if curr_date.day() > date => {
                            if curr_date.month() == 12 {
                                new_date = NaiveDate::from_ymd_opt(curr_date.year()+1, 1, date);
                            } else {
                                new_date = NaiveDate::from_ymd_opt(curr_date.year(), curr_date.month()+1, date);
                            }
                           Some(new_date.unwrap().and_hms_opt(0, 0, 0).unwrap().timestamp())
                        }, // add one month
                        _ => {
                            new_date = NaiveDate::from_ymd_opt(curr_date.year(), curr_date.month(), date);
                            Some(new_date.unwrap().and_hms_opt(0, 0, 0).unwrap().timestamp())
                        }
                    }
                },
                _ if (&string_tuple.1).contains("next") => Some(1),
                _ if MONTHS.keys().any(|&m| m == (&string_tuple.1).split_whitespace()
                                                                        .next()
                                                                        .unwrap()) => Some(1),
                _ => None
            }
            _ => None
        }; 

        if let Err(err_msg) = msg.channel_id.say(&ctx.http, "placeholder").await {
            println!("Error sending message: {err_msg:?}");
        }
    }

    Ok(())
}
