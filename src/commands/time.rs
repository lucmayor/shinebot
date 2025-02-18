use std::arch::x86_64;

#[allow(unreachable_code)]
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::prelude::*;
use serenity::prelude::*;

use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};

use phf::phf_map;

// optimize w/ trait and type conversions
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

static DAYS: phf::Map<&'static str, u8> = phf_map! {
    "mon" => 0, "tue" => 1, "wed" => 2, "thur" => 3,
    "fri" => 4, "sat" => 5, "sun" => 6,

    "monday" => 0, "tuesday" => 1, "wednesday" => 2,
    "thursday" => 3, "friday" => 4, "saturday" => 5,
    "sunday" => 6
};

#[command]
pub async fn todo(ctx: &Context, msg: &Message) -> CommandResult {
    let user_string = &msg.content;

    // note, replace all this with regex: (.+?)\s+(by|in)\s+(.+)

    // strings past user_string[0], iterate until user_string[n] = "by", rest of string arr for dating
    // for dating, switch on date types (minutes / hours / days / months / years)

    // by case -> (specific date) / ('next' 'weekdate') .. we can have reminder time be stock for any given user / we have not
    // in case -> (timeslot) (measurement)

    let mut casing: Option<&str> = None;
    let curr_date = Utc::now().date_naive();

    let string_tuple: (String, String) = match user_string {
        _ if user_string.contains("by") => user_string.find(" by ").map(|pos| {
            let (before, after) = user_string.split_at(pos);
            let date_part = &after[4..];
            casing = Some("by");
            (before.trim().to_string(), date_part.trim().to_string())
        }),
        _ if user_string.contains("in") => user_string.find(" in ").map(|pos| {
            let (before, after) = user_string.split_at(pos);
            let date_part = &after[4..];
            casing = Some("in");
            (before.trim().to_string(), date_part.trim().to_string())
        }),
        _ => panic!("Unknown format for todo string"),
    }
    .unwrap();

    let date_fix: Option<i64> = match &string_tuple.1.chars().filter(|c| *c == ' ').count() {
        0 => todo!(), // temp
        1 => match &string_tuple.1 {
            // the [5th], [6th], [7th] case
            _ if (&string_tuple.1).contains("the") => {
                let substr = &string_tuple.1[..&user_string.len() - 2]; // 32nd
                let date: u32 = substr.parse::<u32>().unwrap();

                let new_date: Option<NaiveDate>;

                // accidentally reinvented the wheel here. optimizable using Duration
                // also make sure this returning the Some(new_date...timestamp())
                if curr_date.day() > date {
                    if curr_date.month() == 12 {
                        new_date = NaiveDate::from_ymd_opt(curr_date.year() + 1, 1, date);
                    } else {
                        new_date =
                            NaiveDate::from_ymd_opt(curr_date.year(), curr_date.month() + 1, date);
                    }
                    Some(new_date.unwrap().and_hms_opt(0, 0, 0).unwrap().timestamp())
                } else {
                    // add one month
                    new_date = NaiveDate::from_ymd_opt(curr_date.year(), curr_date.month(), date);
                    Some(new_date.unwrap().and_hms_opt(0, 0, 0).unwrap().timestamp())
                }
            }

            // 'this saturday', 'this tuesday'
            _ if (&string_tuple.1).contains("this") => {
                match weekday_difference(curr_date, &string_tuple.1[5..]) {
                    // week from now but it's the same so just ask them to input it like next
                    0 => Some(0), // dno how to handle this error but probably do like {panic!(), some(0)}
                    x if x < 0 => Some(
                        (curr_date + Duration::days(7 + x))
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .timestamp(),
                    ),
                    x => Some(
                        (curr_date + Duration::days(x))
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .timestamp(),
                    ),
                }
            }

            // 'next friday', 'next saturday'
            // basically the same as above + 7 days on each case
            // prob optimizable w/ method above but write method later cuz lazy
            _ if (&string_tuple.1).contains("next") => {
                match weekday_difference(curr_date, &string_tuple.1[5..]) {
                    0 => Some(
                        (curr_date + Duration::days(7))
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .timestamp(),
                    ),
                    x if x < 0 => Some(
                        (curr_date + Duration::days(14 + x))
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .timestamp(),
                    ),
                    x => Some(
                        (curr_date + Duration::days(7 + x))
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .timestamp(),
                    ),
                }
            }

            // 'jan 31st', 'july 2nd', etc..
            _ if MONTHS
                .keys()
                .any(|&m| m == (&string_tuple.1).split_whitespace().next().unwrap()) =>
            {
                // this is so stupid LOL
                // rewrite this part ASAP
                let date_str = string_tuple
                    .1
                    .find(" ")
                    .map(|pos| {
                        let (month, day) = string_tuple.1.split_at(pos);
                        casing = Some(" ");
                        (
                            MONTHS.get(&month.trim().to_string()).unwrap(),
                            day.trim().to_string().parse::<i32>().unwrap(),
                        )
                    })
                    .unwrap();

                let temp = NaiveDate::from_ymd_opt(
                    curr_date.year(),
                    (*date_str.0).try_into().unwrap(),
                    date_str.1.try_into().unwrap(),
                )
                .unwrap();

                // returns
                Some(
                    NaiveDate::from_ymd_opt(
                        {
                            if (curr_date - temp).num_seconds() < 0 {
                                curr_date.year() + 1
                            } else {
                                curr_date.year()
                            }
                        },
                        (*date_str.0).try_into().unwrap(),
                        date_str.1.try_into().unwrap(),
                    )
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .timestamp()
                )
            }

            // error! figure this case out later
            _ => todo!()
        },
        _ => panic!("Unexpected return")
    };

    // add the thing to the thing

    // send confirmation
    if let Err(err_msg) = msg.channel_id.say(&ctx.http, "placeholder").await {
        println!("Error sending message: {err_msg:?}");
    }

    Ok(())
}

fn weekday_difference(nd: NaiveDate, date: &str) -> i64 {
    let curr_weekday: u32 = nd.weekday().num_days_from_monday();

    let next_weekday = match Weekday::try_from(*DAYS.get(date).expect("get weekday")) {
        Ok(day) => day,
        Err(e) => todo!(),
    }
    .num_days_from_monday();

    return (curr_weekday - next_weekday) as i64;
}
