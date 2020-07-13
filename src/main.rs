extern crate serenity;

use serenity::model::channel::*;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use std::fs::File;
use std::io::prelude::*;

struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {}

    fn message(&self, _ctx: Context, _new_message: Message) {
        if _new_message.content == "!bubble" {
            if let Err(message) = _new_message.channel_id.say(&_ctx.http, "teaaa!") {
                println!("Error given with message: {:?}", message);
            }
        }
    }

    fn ready(&self, _ctx: Context, _data_about_bot: Ready) {
        println!("{} is ready", _data_about_bot.user.name);
    }
}

fn main() {
    let mut file = File::open(".token").unwrap();
    let mut token = String::new();
    file.read_to_string(&mut token)
        .expect("Token File not found");

    let mut client = Client::new(&token, Handler).expect("Error creating client");

    // Logs to console if there is an error running the bot
    if let Err(message) = client.start() {
        println!("Error: {:?}", message);
    }
    println!("Hello, world!");
}
