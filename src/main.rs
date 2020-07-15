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
    let message = Regex::new(r#""([A-Za-z0-9\D\S]+)""#).unwrap();
    let person = Regex::new(r"@([A-Za-z0-9\D\S])[^\s]+").unwrap();
    let date = Regex::new(r"([0-9]+\s?)(min?s|hr?s|day?s|week?s)+").unwrap();

    let incoming_message = msg.content.replace("~remind", "");
    let remind_message = message.captures(&incoming_message).unwrap();
    let remind_person = person.captures(&incoming_message).unwrap();
    let remind_date = date.captures(&incoming_message).unwrap();

    // message, person, number (4), unit (mins/hrs)
    println!(
        "{:?} {:?} {:?} {:?}",
        &remind_message[0], &remind_person[0], &remind_date[1], &remind_date[2]
    );

    //let (onemin_tx, onemin_rx) = channel();

    let borrow_time = remind_date[1].parse::<u64>().unwrap();
    let borrow_message: String = remind_message[0].to_owned();
    //let should_alert = false;
    //let mutex = std::sync::Mutex::new(should_alert);
    //let arc = std::sync::Arc::new(mutex);

    let child;
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
    use std::sync::atomic::{AtomicBool, Ordering}; //Ordering

    //let timer = Timer::new();
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
        let tru = AtomicBool::new(false);
        //let mut tru_c = std::sync::Arc::new(&self.tru).clone();
        let mut truc = AtomicBool::new(tru.load(Ordering::Relaxed));

        child = thread::spawn(move || {
            thread::sleep(Duration::from_secs(60 * borrow_time));
            //tru_c.store(true, Ordering::Relaxed);
            *truc.get_mut() = true;
        });

        child.join().unwrap();
        /*         if truc.load(Ordering::Relaxed) {
        } */
        msg.reply(ctx, format!("{:?}", borrow_message))?;
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
