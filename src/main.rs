#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std::env;
// use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

use serenity::prelude::*;
use serenity::model::gateway::Ready;
use serenity::model::channel::{Message, Reaction};

mod schema;

struct Handler;

// struct User {
//     id: serenity::model::id::UserId,
//     languages: HashSet<String>,
// }

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

impl EventHandler for Handler {
    fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {

    }

    fn message(&self, _ctx: Context, _message: Message) {

    }

    fn ready(&self, _ctx: Context, bot_status: Ready) {
        println!("{} is ready", bot_status.user.name);
    }
}


fn main() {
    // let connection = establish_connection();

    // let connection = establish_connection();

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
