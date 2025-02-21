#[allow(unreachable_code)]
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::*;

use chrono::{
    Datelike, Duration, Month, NaiveDate, TimeDelta, Utc, Weekday,
};

use regex::{Captures, Regex};

struct FixDate(i64);

#[command]
#[aliases("todo")]
pub async fn todo(ctx: &Context, msg: &Message) -> CommandResult {
    let input = &msg.content;

    let reg: Regex = Regex::new(r"(.?\!do)\s(.+?)\s+(by|in)\s+(.+)").unwrap();
    let captures: Captures<'_> = reg.captures(&input).unwrap();
    let operand: &str = captures.get(2).unwrap().as_str();
    let date: &str = captures.get(3).unwrap().as_str();

    let current_date = Utc::now(); // impl the user
                                   // this will need to be changed for implementation of timezone db

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

                            (current_date + {
                                if casing[0] == "next" {
                                    Duration::from(TimeDelta::days(7))
                                } else {
                                    Duration::from(TimeDelta::days(0))
                                }
                            } + Duration::from(TimeDelta::days(weekday_dfm.into())))
                                .timestamp()
                        }
                        _ if matches!(casing[0].parse::<Month>(), Ok(mon)) => {
                            // idk if this passes.
                            let day = (&casing[1][..2]).parse::<i64>().expect("Invalid date");
                            let mon = casing[0].parse::<Month>().expect("Month errored randomly");

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
                _ => todo!(),
            }
        }
        _ => {
            panic!("Invalid operand for process.")
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
    mon >= current.month() && day > current.day()
}

fn build_date_str(year: i32, month: i32, day: i32) -> String {
    year.to_string() + "-" + &month.to_string() + "-" + &day.to_string()
}
