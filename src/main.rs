#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate serenity;

use diesel::prelude::*;
use dotenv::dotenv;

use std::env;

use serenity::model::channel::{Message, Reaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod schema;
use schema::{match_admin, match_groups, match_responses, user};

struct Handler;

type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Insertable)]
#[table_name = "user"]
struct NewUser<'a> {
    discord_id: i32,
    languages: &'a str,
    pool_state: i32,
}

#[derive(Queryable)]
struct User {
    id: i32,
    discord_id: i32,
    languages: String,
    pool_state: i32,
}

#[derive(Insertable)]
#[table_name = "match_admin"]
struct NewMatchAdmin {
    status: bool,
    questions: Vec<String>,
    user_id: i32,
}

#[derive(Queryable)]
struct MatchAdmin {
    id: i32,
    status: bool,
    questions: Vec<String>,
    user_id: i32,
}

#[derive(Insertable)]
#[table_name = "match_responses"]
struct NewMatchResponses {
    answers: Vec<bool>,
    match_id: i32,
    user_id: i32,
}

#[derive(Queryable)]
struct MatchResponses {
    id: i32,
    answers: Vec<bool>,
    match_id: i32,
    user_id: i32,
}

#[derive(Insertable)]
#[table_name = "match_groups"]
struct NewMatchGroups {
    members: Vec<i32>,
    match_id: i32,
}

#[derive(Queryable)]
struct MatchGroups {
    id: i32,
    members: Vec<i32>,
    match_id: i32,
}

struct PooledConnection(Pool);

impl TypeMapKey for PooledConnection {
    type Value = Pool;
}

fn is_user_exist(id: &u64, connection_pool: &Pool) -> bool {
    use diesel::expression::exists;
    use schema::user::dsl as user_dsl;

    diesel::select(exists::exists(
        schema::user::dsl::user.filter(user_dsl::discord_id.eq(*id as i32)),
    ))
    .get_result::<bool>(&connection_pool.get().unwrap())
    .unwrap()
}

fn insert_user(id: &u64, languages: &String, connection_pool: &Pool) {
    // NewUser is the struct used for inserting into the database
    diesel::insert_into(schema::user::dsl::user)
        .values(NewUser {
            discord_id: *id as i32,
            languages: languages,
            // pool_state = 0, 1, 2 for doing nothing, generating pool, joining pool
            pool_state: 0 as i32,
        })
        .execute(&connection_pool.get().unwrap())
        .unwrap();
}

fn start_pool(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("starting pool"));
}

fn join_pool(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("joining pool"));
}

fn check_pool(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("checking pool"));
}

fn skrt_pool(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("leaving pool"));
}

fn match_pool(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("generating matches"));
}

impl EventHandler for Handler {
    fn reaction_add(&self, context: Context, add_reaction: Reaction) {}

    fn message(&self, context: Context, message: Message) {
        let mut data = context.data.write();
        let connection_pool = data.get_mut::<PooledConnection>().unwrap();

        // Sample parsing of message
        let message_tokens: Vec<&str> = message.content.split(" ").collect();
        let message_author_id = message.author.id.0;

        if message_tokens[0] == "!utilbot" {
            if message_tokens[1] == "pool" {
                match message_tokens[2] {
                    "start" => start_pool(&context, &message, &message_tokens),
                    "join" => join_pool(&context, &message, &message_tokens),
                    "check" => check_pool(&context, &message, &message_tokens),
                    "skrt" => skrt_pool(&context, &message, &message_tokens),
                    "match" => match_pool(&context, &message, &message_tokens),
                    _ => println!("Bad pool command"),
                }
            }

            // Test existance of message sender
            if is_user_exist(&message_author_id, connection_pool) {
                println!(
                    "You're already in the database and your ID is {}",
                    message_author_id
                );
            // TODO: Add query here to verify that user has been added
            // Insert new user that sent the message
            } else {
                insert_user(&message.author.id.0, &message.content, connection_pool);
                println!(
                    "You've been added to the database. Your ID is {}",
                    message_author_id
                )
            }
        }
        // Test existance of random user_id and print result
        let test2: bool = is_user_exist(&032458097234, connection_pool);
        // Print results
        println!("UserID: {} Exists: {}", 1203948799, test2);
    }

    fn ready(&self, context: Context, bot_status: Ready) {
        println!("{} is ready", bot_status.user.name);
    }
}

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let pool: Pool = diesel::r2d2::Pool::new(diesel::r2d2::ConnectionManager::new(database_url))
        .expect("Failed to build pool.");
    let mut client = Client::new(env::var("DISCORD_TOKEN").expect("Missing token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        serenity::framework::standard::StandardFramework::new()
            .configure(|c| c.prefix("~").allow_dm(true)),
    );
    {
        let mut data = client.data.write();
        data.insert::<PooledConnection>(pool);
    }
    if let Err(why) = client.start() {
        println!("Error starting the Discord client: {:?}", why);
    }
}
