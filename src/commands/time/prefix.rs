#[allow(unreachable_code)]
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::*;

use anyhow::Result;

use chrono::{Datelike, Duration, Month, NaiveDate, TimeDelta, Utc, Weekday};
use sqlx::{Pool, Sqlite};

use crate::DatabaseContainer;

use regex::{Captures, Regex};

struct FixDate(i64);

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
#[aliases("todo")]
pub async fn todo(ctx: &Context, msg: &Message) -> CommandResult {
    let input = &msg.content;

    let data = ctx.data.read().await;
    let db = data.get::<DatabaseContainer>().expect("Couldn't find db");

    let reg: Regex = Regex::new(r"(.?\!do)\s(.+?)\s+(by|in)\s+(.+)").unwrap();
    let captures: Captures<'_> = reg.captures(&input).unwrap();
    let operand: &str = captures.get(2).unwrap().as_str();
    let date: &str = captures.get(3).unwrap().as_str();

    let current_date = Utc::now(); // implement the user
                                   // this will need to be changed for implementation of timezone db
                                   // presently checks for UTC

    let calc_timestamp: i64 = match operand {
        "in" => {
            let dur: u64 = humantime::parse_duration(date).unwrap().as_secs();
            println!("Duration: {:?}", dur);

            // construct timestamp
            let time = (current_date + TimeDelta::try_seconds(dur.try_into().unwrap()).unwrap())
                .timestamp();
            dbg!(time);

            todo!()
        }
        "by" => {
            match date.chars().filter(|c| *c == ' ').count() {
                0 => {
                    let date_reg: Regex = Regex::new(r"(\d{4}|\d{2})-(\d{2})-?(\d{2})?").unwrap();
                    let date_captures: Captures<'_> = date_reg.captures(&date).unwrap();

                    // case
                    match date_captures.get(0).unwrap().as_str().chars().count() {
                        4 => {
                            let year: i32 = date_captures
                                .get(0)
                                .unwrap()
                                .as_str()
                                .parse::<i32>()
                                .unwrap();
                            let month: i32 = date_captures
                                .get(1)
                                .unwrap()
                                .as_str()
                                .parse::<i32>()
                                .unwrap();
                            let day: i32 = date_captures
                                .get(2)
                                .unwrap()
                                .as_str()
                                .parse::<i32>()
                                .unwrap();

                            FixDate::from(build_date_str(year, month, day)).0
                        }
                        2 => {
                            let month: i32 = date_captures
                                .get(0)
                                .unwrap()
                                .as_str()
                                .parse::<i32>()
                                .unwrap();
                            let day: i32 = date_captures
                                .get(1)
                                .unwrap()
                                .as_str()
                                .parse::<i32>()
                                .unwrap();

                            FixDate::from(build_date_str(
                                {
                                    if is_after(month as u32, day as u32, current_date.date_naive())
                                    {
                                        current_date.year()
                                    } else {
                                        current_date.year() + 1
                                    }
                                },
                                month,
                                day,
                            ))
                            .0
                        }
                        _ => {
                            panic!("Regex failed.")
                        }
                    }
                }
                1 => {
                    let casing: Vec<&str> = date.split(' ').collect();

                    match casing[0] {
                        "this" | "the" | "next" => {
                            let weekday_current: Weekday = current_date.weekday();

                            let weekday_dfm: u32 = match casing[1].trim().parse::<Weekday>() {
                                Ok(wkdy) => wkdy.days_since(weekday_current),
                                Err(_) => panic!("Invalid weekday"),
                            };

                            (current_date
                                + {
                                    if casing[0] == "next" {
                                        Duration::from(TimeDelta::days(7))
                                    } else {
                                        Duration::from(TimeDelta::days(0))
                                    }
                                }
                                + Duration::from(TimeDelta::days(weekday_dfm.into())))
                            .timestamp()
                        }
                        mon_str if mon_str.is_month() => {
                            // idk if this passes.
                            let day = (&casing[1][..2]).parse::<i64>().expect("Invalid date");
                            let mon = mon_str
                                .parse::<Month>()
                                .expect("Month parse failed after passing test");

                            FixDate::from(build_date_str(
                                {
                                    if is_after(
                                        mon.number_from_month(),
                                        day as u32,
                                        current_date.date_naive(),
                                    ) {
                                        current_date.year()
                                    } else {
                                        current_date.year() + 1
                                    }
                                },
                                mon.number_from_month().try_into().unwrap(),
                                day.try_into().unwrap(),
                            ))
                            .0
                        }
                        _ => panic!("Invalid input for date!"),
                    }
                }
                _ => panic!("Invalid input within date string"),
            }
        }
        _ => {
            panic!("Invalid operand for process.")
        }
    };

    let task_name = captures.get(1).unwrap().as_str();

    match add_item(task_name, calc_timestamp, db.clone(), msg.author.id.get()).await {
        Ok(_) => {
            msg.reply(
                &ctx.http,
                format!("Added ({:?}, {:?}) to your to-do list!", task_name, calc_timestamp)
            ).await?;
        },
        Err(e) => {
            msg.reply(
                &ctx.http,
                format!("Error adding ({:?}, {:?}) to your to-do list: {:?}", task_name, calc_timestamp, e)
            ).await?;
        }
    };

    Ok(())
}

impl From<String> for FixDate {
    fn from(val: String) -> Self {
        // String will take form {year}-{month}-{date}
        let date_reg: Regex = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
        let date_captures: Captures<'_> = date_reg.captures(&val).unwrap();

        FixDate(
            NaiveDate::from_ymd_opt(
                date_captures
                    .get(0)
                    .unwrap()
                    .as_str()
                    .parse::<i32>()
                    .unwrap(),
                date_captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap(),
                date_captures
                    .get(2)
                    .unwrap()
                    .as_str()
                    .parse::<u32>()
                    .unwrap(),
            )
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .timestamp(),
        )
    }
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

fn build_date_str(year: i32, month: i32, day: i32) -> String {
    year.to_string() + "-" + &month.to_string() + "-" + &day.to_string()
}

async fn add_item(task: &str, timestamp: i64, db: Pool<Sqlite>, user_id: u64) -> Result<()> {
    sqlx::query!("INSERT INTO tasks (user_id, task_desc, time_stamp) VALUES ('$1', '$2', '$3');")
        .bind(user_id as i64)
        .bind(task)
        .bind(timestamp)
        .execute(&db)
        .await?;

    // add logic for tcp server ping to alert rebuilder

    Ok(())
}
