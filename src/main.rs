//use chrono;
/* #[macro_use]
extern crate lazy_static; */
extern crate eventual;

use serenity::client::Client;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler};

//use eventual::Timer;
use regex::Regex;
//use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use std::fs::File;
use std::io::prelude::*;

#[group]
#[commands(ping, remind)]
struct General;

//use std::env;
//use std::result::Result;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!online" {
            ctx.online();
        }
    }
}

/* mod private {
    pub const TOKEN: &'static str = "";
} */

fn main() {
    // Login with a bot token from the environment
    let mut file = File::open(".token").unwrap();
    let mut token = String::new();

    file.read_to_string(&mut token)
        .expect("Token File not found");

    //let mut client = Client::new(private::TOKEN, Handler).expect("Error creating client");
    let mut client = Client::new(&token, Handler).expect("Error creating client");

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
            .group(&GENERAL_GROUP),
    );

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!")?;
    //let command = msg.content.replace("~ping", "");
    //let cache = cache.read();
    Ok(())
}

/* #[derive(Clone, Copy)]
enum regexmatches {
    incoming,
    message,
    person,
    date,
} */

#[command]
fn remind(ctx: &mut Context, msg: &Message) -> CommandResult {
    //let remind_message = lazy_static::REMIND_MESSAGE
    let message = Regex::new(r"\b([A-Za-z0-9\D\S]+)\b").unwrap();
    //let person = Regex::new(r"@([A-Za-z0-9\D\S])[^\s]+").unwrap();
    let date = Regex::new(r"([0-9]+)\s?:?([0-9]+)?\s?(min?s|hr?s|day?s|week?s|am|pm)+").unwrap(); // ([0-9]+)?\s(min?s|hr?s|day?s|week?s)+

    //let specific_date = Regex::new(r"([0-9]):([0-9]+)\s?([A-Za-z]+)").unwrap();
    let incoming_message = msg.content.replace("~remind", "");
    let remind_message = message.captures(&incoming_message).unwrap();
    //let remind_person = person.captures(&incoming_message).unwrap();
    let remind_date = date.captures(&incoming_message).unwrap();

    // message, person, number (4), unit (mins/hrs)
    /* println!(
        "{:?} {:?} {:?} {:?}",
        &remind_message[0], &remind_person[0], &remind_date[1], &remind_date[2]
    ); */

    //let (onemin_tx, onemin_rx) = channel();

    let borrow_time = remind_date[1].parse::<u64>().unwrap();
    let borrow_message: String = remind_message[0].to_owned();
    //let should_alert = false;
    //let mutex = std::sync::Mutex::new(should_alert);
    //let arc = std::sync::Arc::new(mutex);

    let child;
    let should_notify;
    /*     {
        let arc = arc.clone();
        child = thread::spawn(move || {
            thread::sleep(Duration::from_secs(60 * borrow_time));
            let mut guard = arc.lock().unwrap();
            *guard = true;
            println!("{:?}", *guard);
            //should_alert = true;
            onemin_tx.send("PING").unwrap(); //format!("{:?}", borrow_message)
        });
    } */
    println!("{:?}", &remind_date[3]);
    use chrono::{Timelike, Utc};
    use std::sync::atomic::{AtomicBool, Ordering}; //Ordering
                                                   //let timer = Timer::new();
                                                   //use std::time::{Duration, Instant};
    let now = Utc::now();
    let (is_pm, hour) = now.hour12();
    if &remind_date[2] == "mins" || &remind_date[2] == "min" {
        //let min_ticks = timer.interval_ms(1000).iter();
        /* loop {
            let _ = onemin_rx.try_recv().map(|reply| println!("{}", reply));
            let guard = arc.lock().unwrap();

            if *guard {
                println!("LEAVING!");
                break;
            }
        } */

        //let tru = std::sync::Arc::new(AtomicBool::new(false));
        should_notify = AtomicBool::new(false);
        //let mut tru_c = std::sync::Arc::new(&self.tru).clone();
        let mut should_notify_clone = AtomicBool::new(should_notify.load(Ordering::Relaxed));

        child = thread::spawn(move || {
            thread::sleep(Duration::from_secs(60 * borrow_time));
            //tru_c.store(true, Ordering::Relaxed);
            *should_notify_clone.get_mut() = true;
        });

        child.join().unwrap();
        msg.reply(ctx, format!("{:?}", borrow_message))?;

    /*         if truc.load(Ordering::Relaxed) {
    } */
    } else if &remind_date[2] == "hrs" || &remind_date[2] == "hr" {
        should_notify = AtomicBool::new(false);
        let mut should_notify_clone = AtomicBool::new(should_notify.load(Ordering::Relaxed));
        child = thread::spawn(move || {
            thread::sleep(Duration::from_secs(3600 * borrow_time));
            *should_notify_clone.get_mut() = true;
        });

        child.join().unwrap();
        msg.reply(ctx, format!("{:?}", borrow_message))?;
    } else if &remind_date[2] == "days" || &remind_date[2] == "day" {
        should_notify = AtomicBool::new(false);
        let mut should_notify_clone = AtomicBool::new(should_notify.load(Ordering::Relaxed));
        child = thread::spawn(move || {
            thread::sleep(Duration::from_secs(86400 * borrow_time));
            *should_notify_clone.get_mut() = true;
        });

        child.join().unwrap();
        msg.reply(ctx, format!("{:?}", borrow_message))?;
    } else if &remind_date[2] == "weeks" || &remind_date[2] == "week" {
        should_notify = AtomicBool::new(false);
        let mut should_notify_clone = AtomicBool::new(should_notify.load(Ordering::Relaxed));
        child = thread::spawn(move || {
            thread::sleep(Duration::from_secs(604800 * borrow_time));
            *should_notify_clone.get_mut() = true;
        });

        child.join().unwrap();
        msg.reply(ctx, format!("{:?}", borrow_message))?;
    } else if &remind_date[3] == "pm" {
        let i_hr = remind_date[1].to_string().parse::<u64>().unwrap();
        let hr = hour.to_string().parse::<u64>().unwrap();

        let i_min = remind_date[2].to_string().parse::<u64>().unwrap();
        let m = now.minute().to_string().parse::<u64>().unwrap();

        let mut diff_hr: u64 = 0;
        //println!("{}, {}", i_min, m);
        if i_min > hr {
            diff_hr = i_hr - hr;
        }

        let diff_min = i_min - m;

        let t_secs;
        if diff_hr != 0 {
            t_secs = (diff_hr * 3600) + (diff_min * 60);
        } else {
            t_secs = diff_min * 60;
        }

        let sshould_notify = AtomicBool::new(false);
        let mut should_notify_clone = AtomicBool::new(sshould_notify.load(Ordering::Relaxed));
        let cchild = thread::spawn(move || {
            thread::sleep(Duration::from_secs(t_secs));
            *should_notify_clone.get_mut() = true;
        });

        cchild.join().unwrap();
        msg.reply(ctx, format!("{:?}", borrow_message))?;
    } else if &remind_date[3] == "am" {
        if !is_pm {
            // it's am
            // would calculate normally as if it's the "pm" case
            let i_hr = remind_date[1].to_string().parse::<u64>().unwrap();
            let hr = hour.to_string().parse::<u64>().unwrap();

            let i_min = remind_date[2].to_string().parse::<u64>().unwrap();
            let m = now.minute().to_string().parse::<u64>().unwrap();
            let mut diff_hr: u64 = 0;

            if i_min > hr {
                diff_hr = i_hr - hr;
            }

            let diff_min = i_min - m;

            let t_secs;
            if diff_hr != 0 {
                t_secs = (diff_hr * 3600) + (diff_min * 60);
            } else {
                t_secs = diff_min * 60;
            }

            let sshould_notify = AtomicBool::new(false);
            let mut should_notify_clone = AtomicBool::new(sshould_notify.load(Ordering::Relaxed));
            let cchild = thread::spawn(move || {
                thread::sleep(Duration::from_secs(t_secs));
                *should_notify_clone.get_mut() = true;
            });

            cchild.join().unwrap();
            msg.reply(ctx, format!("{:?}", borrow_message))?;
        } else {
            // it's pm
            // ex. 4:30pm remind at 2:15am
            // convert to 16:30 and 2:15
            let target = 12 - hour.to_string().parse::<u64>().unwrap()
                + remind_date[1].to_string().parse::<u64>().unwrap();
            let min_sub_target = remind_date[2].to_string().parse::<u64>().unwrap() / 60;
            let net_target = target - min_sub_target;
            let net_target_secs = 60 * net_target;

            let i_date_secs;
            if remind_date[1].to_string().parse::<u64>().unwrap() != 12 {
                i_date_secs = (remind_date[1].to_string().parse::<u64>().unwrap() * 3600)
                    + (remind_date[2].to_string().parse::<u64>().unwrap() * 60);
            } else {
                i_date_secs = remind_date[2].to_string().parse::<u64>().unwrap() * 60;
            }

            let diff = net_target_secs - i_date_secs;
            let sshould_notify = AtomicBool::new(false);
            let mut should_notify_clone = AtomicBool::new(sshould_notify.load(Ordering::Relaxed));
            let cchild = thread::spawn(move || {
                thread::sleep(Duration::from_secs(diff));
                *should_notify_clone.get_mut() = true;
            });

            cchild.join().unwrap();
            msg.reply(ctx, format!("{:?}", borrow_message))?;
        }
    }

    /* {
        if &remind_date[2] == "mins" || &remind_date[2] == "min" {
            //let min_ticks = timer.interval_ms(1000).iter();
            loop {
                let _ = onemin_rx.try_recv().map(|reply| println!("{}", reply));
                let guard = arc.lock().unwrap();

                if *guard {
                    println!("LEAVING!");
                    break;
                }
            }
        }
    } */

    /* child.join().unwrap();
    let guard = arc.lock().unwrap();
    println!("RUNNING!");
    if *guard {
        msg.reply(ctx, format!("{:?}", borrow_message))?;
    } */
    //static msg.reply(ctx, format!("{:?}", borrow_message));

    Ok(())
}

/*     static ref REMIND_FORMAT: Regex = Regex::new(
    r#""([A-Za-z0-9\D\S]+)"\s+@([A-Za-z0-9\D\S]+)\s+([0-9]+\s?(min?s|hr?s|day?s|week?s)+)"#
).unwrap(); */
