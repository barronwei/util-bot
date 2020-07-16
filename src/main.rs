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
    languages: Vec<&'a str>,
    pool_state: i32,
}

#[derive(Queryable)]
struct User {
    id: i32,
    discord_id: i32,
    languages: Vec<String>,
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

fn insert_user(id: &u64, languages: Vec<String>, connection_pool: &Pool) {
    // NewUser is the struct used for inserting into the database
    diesel::insert_into(schema::user::dsl::user)
        .values(NewUser {
            discord_id: *id as i32,
            languages: languages.iter().map(AsRef::as_ref).collect(),
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
fn clear_user_languages(id: &u64, connection_pool: &Pool) {
    // User clears the languages
    let empty: Vec<String> = Vec::new();
    diesel::update(schema::user::dsl::user)
        .set(schema::user::dsl::languages.eq(empty))
        .filter(schema::user::dsl::discord_id.eq(*id as i32))
        .execute(&connection_pool.get().unwrap()).ok();
}

fn get_user_languages(user_id: &u64, connection_pool: &Pool) -> Vec<String> {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results: std::vec::Vec<User> = user.
    filter(discord_id.eq(*user_id as i32))
    .load::<User>(&connection)
    .expect("error");
    let return_result = &results[0].languages;
    return_result.to_vec()
}

fn update_user_languages(user_id: &u64, new_languages: Vec<String>, connection_pool: &Pool) {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();

    let results: std::vec::Vec<User> = user.
    filter(discord_id.eq(*user_id as i32))
    .load::<User>(&connection)
    .expect("error");
    
    // Assuming dicord_id unique
    let mut user_languages = &results[0].languages;
    let length = results[0].languages.len();

    for language in new_languages {
        if !user_languages.contains(&language) {
            let mut old = get_user_languages(user_id, connection_pool);
            old.push(language);

            diesel::update(schema::user::dsl::user)
                .set(schema::user::dsl::languages.eq(old))
                .filter(schema::user::dsl::discord_id.eq(*user_id as i32))
                .execute(&connection_pool.get().unwrap()).ok();
        }
    }
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
            if message_tokens.len() < 2 {
                return;
            }
            if message_tokens[1] == "pool" {
                match message_tokens[2] {
                    "start" => start_pool(&context, &message, &message_tokens),
                    "join" => join_pool(&context, &message, &message_tokens),
                    "check" => check_pool(&context, &message, &message_tokens),
                    "skrt" => skrt_pool(&context, &message, &message_tokens),
                    "match" => match_pool(&context, &message, &message_tokens),
                    _ => println!("Bad pool command"),
                }
            } else if message_tokens[1] == "clear" {
                if is_user_exist(&message_author_id, connection_pool) {
                    clear_user_languages(&message_author_id, connection_pool);
                } else {
                    println!("No record present to clear");
                }
            } else if message_tokens[1] == "view" {
                if is_user_exist(&message_author_id, connection_pool) {
                    let languages = get_user_languages(&message_author_id, connection_pool);
                    println!("Languages: ");
                    for language in languages {
                        println!("{}", language);
                    }
                } else {
                    println!("No record present to view");
                }
            } else if message_tokens[1] == "add" {
                if message_tokens.len() < 3 {
                    println!("Not enough arguments");
                    return;
                }
                let cfg_strings: Vec<&str> = message_tokens[2..].to_vec();
                let mut strings_vec: Vec<String> = Vec::new();
                for s in &cfg_strings {
                    strings_vec.push(s.to_string());
                }
                if is_user_exist(&message_author_id, connection_pool) {
                    update_user_languages(&message_author_id, strings_vec, connection_pool);
                } else {
                    insert_user(&message_author_id, strings_vec , connection_pool);
                }
            } else {
                println!("Bad command")
            }
        }
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
