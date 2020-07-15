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
    group_state: i32,
}

#[derive(Queryable)]
<<<<<<< HEAD
struct User {
    id: i32,
    discord_id: i32,
    languages: String,
    group_state: i32,
}

#[derive(Insertable)]
#[table_name = "match_admin"]
struct NewMatchAdmin {
    questions: Vec<String>,
    user_id: i32,
}

#[derive(Queryable)]
struct MatchAdmin {
    id: i32,
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
=======
pub struct User {
    pub id: i32,
    pub discord_id: i32,
    pub languages: String,
>>>>>>> Add query template for users
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
<<<<<<< HEAD
    ))
    .get_result::<bool>(&connection_pool.get().unwrap())
    .unwrap()
=======
    )).get_result::<bool>(&connection_pool.get().unwrap()).unwrap()
>>>>>>> Add query template for users
}

fn insert_user(id: &u64, languages: &String, connection_pool: &Pool) {
    // NewUser is the struct used for inserting into the database
    diesel::insert_into(schema::user::dsl::user)
        .values(NewUser {
            discord_id: *id as i32,
            languages: languages,
            group_state: 0 as i32,
        })
        .execute(&connection_pool.get().unwrap())
        .unwrap();
}

<<<<<<< HEAD
fn start_group(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("starting group"));
}
=======
fn clear_user_languages(id: &u64, connection_pool: &Pool) {
    // User clears the languages
    diesel::update(schema::user::dsl::user)
        .set(schema::user::dsl::languages.eq(""))
        .filter(schema::user::dsl::discord_id.eq(*id as i32))
        .execute(&connection_pool.get().unwrap()).ok();
}

fn add_user_langauges() {}

fn get_user_languages(user_id: &u64, connection_pool: &Pool) -> String {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results: std::vec::Vec<User> = user.
    filter(discord_id.eq(*user_id as i32))
    .load::<User>(&connection)
    .expect("error");
    let return_result = &results[0].languages;
    return_result.to_string()
}

fn update_user_languages(user_id: &u64, connection_pool: &Pool) {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results: std::vec::Vec<User> = user.
    filter(discord_id.eq(*user_id as i32))
    .load::<User>(&connection)
    .expect("error");
}


impl EventHandler for Handler {
    fn reaction_add(&self, context: Context, add_reaction: Reaction) {
>>>>>>> Add query template for users

fn join_group(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("joining group"));
}

fn check_group(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("checking group"));
}

fn skrt_group(context: &Context, message: &Message, message_tokens: &Vec<&str>) {
    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content("leaving group"));
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
            if message_tokens[1] == "group" {
                match message_tokens[2] {
                    "start" => start_group(&context, &message, &message_tokens),
                    "join" => join_group(&context, &message, &message_tokens),
                    "check" => check_group(&context, &message, &message_tokens),
                    "skrt" => skrt_group(&context, &message, &message_tokens),
                    _ => println!("Bad group command"),
                }
            }

            
            // Test existance of message sender
            if is_user_exist(&message_author_id, connection_pool) {
<<<<<<< HEAD
                println!(
                    "You're already in the database and your ID is {}",
                    message_author_id
                );
            // TODO: Add query here to verify that user has been added
            // Insert new user that sent the message
=======
                println!("You're already in the database and your ID is {}", message_author_id);
                // TODO: Add query here to verify that user has been added
                
                // Insert new user that sent the message
>>>>>>> Add query template for users
            } else {
                insert_user(&message.author.id.0, &message.content, connection_pool);
                println!(
                    "You've been added to the database. Your ID is {}",
                    message_author_id
                )
            }
            if message_tokens[1] == "clear" {
                if !is_user_exist(&message_author_id, connection_pool) { return; }
                else {

                }
            } else {
                // Query update
                println!("{}",get_user_languages(&message_author_id, connection_pool));
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
