#[allow(unreachable_code)]
use serenity::framework::standard::macros::command;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::framework::standard::CommandResult;

use anyhow::Result;

use chrono::{
    DateTime, Datelike, Days, Duration, Local, Month, NaiveDate, NaiveTime, TimeDelta, TimeZone, Timelike, Weekday
};
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite};

use crate::DatabaseContainer;
use humantime::parse_duration;
use regex::{Captures, Match, Regex};
use text2num::{replace_numbers_in_text, Language};

trait MonthValidation {
    fn is_month(&self) -> bool;
}

impl MonthValidation for &str {
    fn is_month(&self) -> bool {
        match self.parse::<Month>() {
            Ok(_val) => true,
            Err(_e) => false,
        }
    }
}

#[command]
#[aliases("todo", "do")]
pub async fn todo(ctx: &Context, msg: &Message) -> CommandResult {
    let input = &msg.content;

    let data = ctx.data.read().await;
    let db = data
        .get::<DatabaseContainer>()
        .expect("Couldn't find the DB!");

    let cmd_regex: Regex = Regex::new(r"(.?\!todo|!do)\s(.+?)\s+(by|in)\s+(.+)").unwrap();
    let captures: Captures<'_> = cmd_regex.captures(&input).unwrap();

    let task_name: &str = captures.get(2).unwrap().as_str();
    let operand: &str = captures.get(3).unwrap().as_str();
    let date: &str = captures.get(4).unwrap().as_str();

    dbg!(format!("task {:?} date {:?}", task_name, date));

    let current_date = Local::now();

    let timestamp: i64 = match operand {
        "in" => {
            let dur: u64 = match parse_duration(date) {
                Ok(res) => res.as_secs(),
                Err(_) => {
                    // don't have to err this but this is fine for now, replacenums just does shit without worry
                    let en = Language::english();
                    let str: &str = &replace_numbers_in_text(date, &en, 0.0);
                    parse_duration(str).unwrap().as_secs()
                }
            };

            // construct timestamp
            let time = (current_date + TimeDelta::try_seconds(dur.try_into().unwrap()).unwrap())
                .timestamp();

            time
        }
        "by" => {
            match date.chars().filter(|c| *c == ' ').count() {
                0 => match &date.parse::<Weekday>() {
                    // basic weekday
                    Ok(_) => weekday(current_date.into(), date, false).await,
                    // full date
                    Err(_) => {
                        let (ind, _) = date.char_indices().rev().nth(1).expect({
                            let _ = err_reply(msg, ctx, "Couldn't prompt the time/date");
                            "Couldn't prompt time"
                        });

                        // use 0 just because i don't care
                        let num = match date[..ind].parse::<u32>() {
                            Ok(i) => i,
                            Err(_) => 0,
                        };

                        match &date[ind..] {
                            "pm" => time_rem(num + 12).await,
                            "am" => time_rem(num).await,
                            "st" | "nd" | "th" | "rd" => {
                                single_date_case(
                                    current_date,
                                    date[..ind].parse::<u32>().unwrap(),
                                )
                                .await
                            }
                            _ => full_date(date).await,
                        }
                    }
                },
                1 => {
                    let casing: Vec<&str> = date.split(' ').collect();

                    match casing[0] {
                        "this" | "the" => {
                            if let Ok(_) = casing[1].parse::<Weekday>() {
                                weekday(current_date.into(), casing[1], false).await
                            } else {
                                if let Some((i, _)) = casing[1].char_indices().rev().nth(1) {
                                    let num = casing[1][..i].parse::<u32>().unwrap();
                                    match &casing[1][i..] {
                                        "st" | "nd" | "th" | "rd" => {
                                            single_date_case(current_date, num).await
                                        }
                                        _ => {
                                            let _ = err_reply(
                                                msg,
                                                ctx,
                                                "Bad suffix for time/date found",
                                            );
                                            panic!("Bad suffix for time/date found")
                                        }
                                    }
                                } else {
                                    let _ = err_reply(
                                        msg,
                                        ctx,
                                        "Bad general input for time/date found",
                                    );
                                    panic!("Bad general input for time/date found")
                                }
                            }
                        }
                        "next" => weekday(current_date.into(), casing[1], true).await,
                        mon_str if mon_str.is_month() => {
                            let mut day_init = casing[1].chars();
                            if casing[1].len() >= 3 {
                                day_init.next_back();
                                day_init.next_back();
                            }

                            let day = day_init
                                .as_str()
                                .parse::<i32>()
                                .expect("Failed day collection");
                            let mon = mon_str
                                .parse::<Month>()
                                .expect("Month parse failed after passing precursor test");

                            build_time(mon.number_from_month(), day as u32, None).await
                        }
                        _ => {
                            let _ = err_reply(msg, ctx, "Invalid input for date!").await;
                            panic!("Invalid input for date")
                        }
                    }
                }
                _ => {
                    let _ = err_reply(msg, ctx, "Invalid input within date str!").await;
                    panic!("Invalid input within date string.")
                }
            }
        }
        _ => {
            let _ = err_reply(msg, ctx, "Invalid operand for command.").await;
            panic!("Invalid operand for process.")
        }
    };

    dbg!(timestamp);
    match add(task_name, timestamp, db.clone(), msg.author.id.get()).await {
        Ok(_) => {
            msg.reply(
                &ctx.http,
                format!(
                    "Added ({:?}, <t:{:?}>) to your to-do list!",
                    task_name, timestamp
                ),
            )
            .await?;
        }
        Err(e) => {
            msg.reply(
                &ctx.http,
                format!(
                    "Error adding ({:?}, <t:{:?}>) to your to-do list: {:?}",
                    task_name, timestamp, e
                ),
            )
            .await?;
        }
    };

    Ok(())
}

async fn time_rem(time: u32) -> i64 {
    let curr_time = Local::now();
    if curr_time.time().hour() > time {
        Local::now()
            .checked_add_days(Days::new(1))
            .unwrap()
            .with_time(NaiveTime::from_hms_opt(time, 0, 0).expect("Failed to make naivetime"))
            .unwrap()
            .timestamp()
    } else {
        Local::now()
            .with_time(NaiveTime::from_hms_opt(time, 0, 0).expect("Failed to make naivetime"))
            .unwrap()
            .timestamp()
    }
}

async fn single_date_case(curr: DateTime<Local>, date: u32) -> i64 {
    if curr.day() > date {
        build_time(curr.month() + 1, date, {
            if curr.month() + 1 > 12 {
                Some(curr.year() + 1)
            } else {
                Some(curr.year())
            }
        })
        .await
    } else {
        build_time(curr.month(), date, Some(curr.year())).await
    }
}

async fn weekday(curr: DateTime<Local>, week: &str, next: bool) -> i64 {
    let weekday_dfm: u32 = match week.trim().parse::<Weekday>() {
        Ok(wkdy) => wkdy.days_since(curr.weekday()),
        Err(_e) => {
            // find a way to print error
            panic!("Invalid weekday");
        }
    };

    (curr
        + {
            if next {
                Duration::from(TimeDelta::days(7))
            } else {
                // need this else block apparently
                Duration::from(TimeDelta::days(0))
            }
        }
        + Duration::from(TimeDelta::days(weekday_dfm.into())))
    .timestamp()
}

async fn full_date(date: &str) -> i64 {
    let date_reg: Regex = Regex::new(r"(\d{4}|\d{2})-(\d{2}|\d{1})-?(\d{2}|\d{1})?").unwrap();
    let date_captures: Captures<'_> = date_reg.captures(&date).unwrap();
    dbg!(&date_captures);

    match date_captures.get(1).unwrap().as_str().chars().count() {
        // year, mon, day
        4 => {
            interim_bt(
                date_captures.get(2).unwrap(),
                date_captures.get(3).unwrap(),
                Some(date_captures.get(1).unwrap()),
            )
            .await
        }
        // mon, day
        2 => {
            interim_bt(
                date_captures.get(1).unwrap(),
                date_captures.get(2).unwrap(),
                None,
            )
            .await
        }
        _ => {
            // find a way to print error
            panic!("Regex failed.");
        }
    }
}

async fn interim_bt(m: Match<'_>, d: Match<'_>, y: Option<Match<'_>>) -> i64 {
    build_time(
        m.as_str().parse::<u32>().unwrap(),
        d.as_str().parse::<u32>().unwrap(),
        {
            match y {
                Some(y) => Some(y.as_str().parse::<i32>().unwrap()),
                None => None,
            }
        },
    )
    .await
}

async fn build_time(month: u32, day: u32, year: Option<i32>) -> i64 {
    let init = match year {
        Some(val) => NaiveDate::from_ymd_opt(val, month, day)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        None => NaiveDate::from_ymd_opt(
            {
                if is_after(month, day, Local::now().date_naive()) {
                    Local::now().year()
                } else {
                    Local::now().year() + 1
                }
            },
            month,
            day,
        )
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap(),
    };

    Local.from_local_datetime(&init).unwrap().timestamp()
}

async fn err_reply(msg: &Message, ctx: &Context, err: &str) -> CommandResult {
    msg.reply(&ctx.http, format!("Error with `!todo`: {:?}", err))
        .await?;

    Ok(())
}

fn is_after(mon: u32, day: u32, current: NaiveDate) -> bool {
    if mon == current.month() {
        day > current.day()
    } else if mon > current.month() {
        true
    } else {
        false
    }
}

async fn add(
    task: &str,
    ts: i64,
    db: Pool<Sqlite>,
    uid: u64,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let conv = uid as i64;
    // this error is sometimes raised but it's just rust-analyzer tripping out
    sqlx::query!(
        r#"INSERT INTO tasks (user_id, task_desc, time_stamp) VALUES (?1, ?2, ?3);"#,
        conv,
        task,
        ts
    )
    .execute(&db)
    .await

    // add logic for tcp server ping to alert rebuilder
}
