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
use schema::*;

struct Handler;

type Pool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

#[derive(Insertable)]
#[table_name = "user"]
struct NewUser<'a> {
    discord_id: i32,
    discord_id_full: i64,
    languages: Vec<&'a str>,
    pool_state: i32,
}

#[derive(Queryable)]
struct User {
    id: i32,
    discord_id: i32,
    discord_id_full: i64,
    languages: Vec<String>,
    pool_state: i32,
}

#[derive(Insertable)]
#[table_name = "match_admin"]
struct NewMatchAdmin {
    user_id: i32,
    status: bool,
    group_size: i32,
}

#[derive(Queryable)]
struct MatchAdmin {
    id: i32,
    user_id: i32,
    status: bool,
    group_size: i32,
}

#[derive(Insertable)]
#[table_name = "pool_questions"]
struct NewPoolQuestions {
    pool_id: i32,
    question: String,
}

#[derive(Queryable)]
struct PoolQuestions {
    id: i32,
    pool_id: i32,
    question: String,
}

#[derive(Insertable)]
#[table_name = "match_responses"]
struct NewMatchResponses {
    match_id: i32,
    user_id: i32,
}

#[derive(Queryable)]
struct MatchResponses {
    id: i32,
    match_id: i32,
    user_id: i32,
}

#[derive(Insertable)]
#[table_name = "pool_responses"]
struct NewPoolResponses {
    response_id: i32,
    answer: String,
}

#[derive(Queryable)]
struct PoolResponses {
    id: i32,
    response_id: i32,
    answer: String,
}

#[derive(Insertable)]
#[table_name = "match_groups"]
struct NewMatchGroups {
    match_id: i32,
    members: Vec<i32>,
}

#[derive(Queryable)]
struct MatchGroups {
    id: i32,
    match_id: i32,
    members: Vec<i32>,
}

struct PooledConnection(Pool);

#[derive(PartialEq)]
enum ReplyType {
    Add,
    Clear,
    GetHelp,
}

impl TypeMapKey for PooledConnection {
    type Value = Pool;
}

fn get_proficient_users(connection_pool: &Pool, language: &String) -> Vec<i64> {
    use schema::user::dsl::*;
    let results = user
        .filter(languages.contains(vec![language]))
        .load::<User>(&connection_pool.get().unwrap())
        .unwrap();

    // Used for debugging lololololol
    // for result in results {
    //     println!("{} knows {}", result.discord_id_full, language);
    // }
    results
        .iter()
        .map(|result| result.discord_id_full)
        .collect()
}

fn reply_for_add_view (
    uid: &u64,
    message: &Message,
    context: &Context,
    connection_pool: &Pool
) {
    let langs = get_user_languages(uid, connection_pool);
    let tag = message.author.mention();
    let mut message_return;
    if langs.len() == 0 {
        message_return = format!("Hey {}, you haven't added anything :cry: \n", tag);
    } else {
        message_return = format!("Hey {}, you've added\n", tag);
    }

    let channel_id = message.channel_id;
    let connected = &langs.join("\n");
    message_return.push_str(connected);
    // Can also use channel_id.say or somehting like tath
    channel_id.send_message(&context.http, |m| {
        m.content(message_return);
        m
    }).unwrap();

}

fn ping_proficient_users (
    uid: &u64,
    recipient_ids: &Vec<i64>,
    lang: &String,
    question: &Vec<&str>,
    message: &Message,
    context: &Context
) {
    let guild_id = message.guild_id.unwrap();
    if recipient_ids.len() == 0 {  
        let return_string = format!("Unfortunaetly, no one has added {}", lang);
        let channel_id = message.channel_id;
        channel_id.send_message(&context.http, |m| {
            m.content(return_string);
        m
        }).unwrap();
    }
    for recipient in recipient_ids.iter() {
        let member = context.http.get_member(guild_id.0, *recipient as u64).unwrap();
        let user_id = member.user_id();
        let private_channel = user_id.create_dm_channel(&context.http).unwrap();
        let connected_question = question.join(" ");
        let user_line = format!("A fellow with userID {} ", message.author.tag());
        let help_line = format!("needs help with {}\n", *lang);
        let question_line = format!("Their question is: {}", connected_question);
        let return_string = format!("{}{}{}",user_line,help_line,question_line);

        private_channel.send_message(&context.http, |m| {
            m.content(return_string);
        m
        }).unwrap();
    }
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
            discord_id_full: *id as i64,
            languages: languages.iter().map(AsRef::as_ref).collect(),
            // pool_state = 0, 1, 2 for doing nothing, generating pool, joining pool
            pool_state: 0 as i32,
        })
        .execute(&connection_pool.get().unwrap())
        .unwrap();
}

fn get_user_id(uid: &u64, connection_pool: &Pool) -> i32 {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results = user
        .filter(discord_id.eq(*uid as i32))
        .load::<User>(&connection)
        .expect("error getting user");
    results[0].id
}

fn get_latest_started_pool(uid: &u64, connection_pool: &Pool) -> i32 {
    use schema::match_admin::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results = match_admin
        .filter(user_id.eq(get_user_id(&uid, &connection_pool)))
        .order(id.desc())
        .load::<MatchAdmin>(&connection)
        .expect("error getting latest started pool");
    results[0].id
}

fn get_latest_joined_pool(uid: &u64, connection_pool: &Pool) -> i32 {
    use schema::match_responses::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results = match_responses
        .filter(user_id.eq(get_user_id(&uid, &connection_pool)))
        .order(id.desc())
        .load::<MatchResponses>(&connection)
        .expect("error getting latest joined pool");
    results[0].id
}
fn clear_user_languages(id: &u64, connection_pool: &Pool) {
    // User clears the languages
    let empty: Vec<String> = Vec::new();
    diesel::update(schema::user::dsl::user)
        .set(schema::user::dsl::languages.eq(empty))
        .filter(schema::user::dsl::discord_id.eq(*id as i32))
        .execute(&connection_pool.get().unwrap())
        .ok();
}

fn get_user_languages(user_id: &u64, connection_pool: &Pool) -> Vec<String> {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results: std::vec::Vec<User> = user
        .filter(discord_id.eq(*user_id as i32))
        .load::<User>(&connection)
        .expect("error");
    let return_result = &results[0].languages;
    return_result.to_vec()
}

fn update_user_languages(user_id: &u64, new_languages: Vec<String>, connection_pool: &Pool) {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();

    let results: std::vec::Vec<User> = user
        .filter(discord_id.eq(*user_id as i32))
        .load::<User>(&connection)
        .expect("error");
    // Assuming dicord_id unique
    let user_languages = &results[0].languages;

    for language in new_languages {
        if !user_languages.contains(&language) {
            let mut old = get_user_languages(user_id, connection_pool);
            old.push(language);

            diesel::update(schema::user::dsl::user)
                .set(schema::user::dsl::languages.eq(old))
                .filter(schema::user::dsl::discord_id.eq(*user_id as i32))
                .execute(&connection_pool.get().unwrap())
                .ok();
        }
    }
}

fn insert_question(uid: &u64, connection_pool: &Pool, text: &String) {
    let connection = connection_pool.get().unwrap();
    diesel::insert_into(schema::pool_questions::dsl::pool_questions)
        .values(NewPoolQuestions {
            pool_id: get_latest_started_pool(&uid, &connection_pool),
            question: text.to_string(),
        })
        .execute(&connection)
        .unwrap();
}

fn insert_response(uid: &u64, connection_pool: &Pool, text: &String) {
    let connection = connection_pool.get().unwrap();
    diesel::insert_into(schema::pool_responses::dsl::pool_responses)
        .values(NewPoolResponses {
            response_id: get_latest_joined_pool(&uid, &connection_pool),
            answer: text.to_string(),
        })
        .execute(&connection)
        .unwrap();
}

fn insert_pool(uid: &u64, connection_pool: &Pool, size: i32) {
    let connection = connection_pool.get().unwrap();
    diesel::insert_into(schema::match_admin::dsl::match_admin)
        .values(NewMatchAdmin {
            user_id: get_user_id(&uid, &connection_pool),
            status: true,
            group_size: size,
        })
        .execute(&connection)
        .unwrap();
}

fn insert_response_header(uid: &u64, connection_pool: &Pool, match_id: i32) {
    let connection = connection_pool.get().unwrap();
    diesel::insert_into(schema::match_responses::dsl::match_responses)
        .values(NewMatchResponses {
            user_id: get_user_id(&uid, &connection_pool),
            match_id,
        })
        .execute(&connection)
        .unwrap();
}

fn get_pool_status(uid: &u64, connection_pool: &Pool) -> i32 {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results = user
        .filter(discord_id.eq(*uid as i32))
        .load::<User>(&connection)
        .expect("error getting pool status");
    results[0].pool_state
}

fn start_pool(
    context: &Context,
    message: &Message,
    message_tokens: &Vec<&str>,
    connection_pool: &Pool,
) {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    if get_pool_status(&message.author.id.0, &connection_pool) != 0 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!(
                "You are in the middle of creating or joining a pool"
            ))
        });
        return;
    }
    if message_tokens.len() < 4 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!(
                "Please use `!utilbot pool start N` for N number of people in a group!"
            ))
        });
        return;
    }

    let group_size = message_tokens[3].parse::<i32>();
    if group_size.is_err() || group_size.unwrap() < 2 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("Please use a group size greater than 1!"))
        });
        return;
    }

    let updated = diesel::update(user.filter(discord_id.eq(message.author.id.0 as i32)))
        .set(pool_state.eq(2))
        .get_result::<User>(&connection);

    if updated.is_err() {
        let _msg = message
            .author
            .direct_message(&context.http, |m| m.content(format!("unknown issue")));
    }

    insert_pool(
        &message.author.id.0,
        &connection_pool,
        message_tokens[3].parse::<i32>().unwrap(),
    );

    let _msg = message
        .author
        .direct_message(&context.http, |m| m.content(format!("Started pool of size {}! Just keep sending me questions individually, and ping me with `done` when you are!", message_tokens[3])));
}

fn join_pool(
    context: &Context,
    message: &Message,
    message_tokens: &Vec<&str>,
    connection_pool: &Pool,
) {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    if get_pool_status(&message.author.id.0, &connection_pool) != 0 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!(
                "You are in the middle of creating or joining a pool"
            ))
        });
        return;
    }
    if message_tokens.len() < 4 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("Please use `!utilbot pool join POOL_ID`!"))
        });
        return;
    }

    if message_tokens[3].parse::<i32>().is_err() || message_tokens[3].parse::<i32>().unwrap() < 1 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("Please use a proper pool id"))
        });
        return;
    }

    let updated = diesel::update(user.filter(discord_id.eq(message.author.id.0 as i32)))
        .set(pool_state.eq(1))
        .get_result::<User>(&connection);

    if updated.is_err() {
        let _msg = message
            .author
            .direct_message(&context.http, |m| m.content(format!("unknown issue")));
    }

    insert_response_header(
        &message.author.id.0,
        &connection_pool,
        message_tokens[3].parse::<i32>().unwrap(),
    );

    // TODO(barronwei): get all questions for pool id and display to user

    use schema::pool_questions::dsl::*;
    let connection = connection_pool.get().unwrap();
    let results = pool_questions
        .filter(pool_id.eq(message_tokens[3].parse::<i32>().unwrap()))
        .load::<PoolQuestions>(&connection)
        .expect("error getting pool questions");

    for result in results.into_iter() {
        let _msg = message
            .author
            .direct_message(&context.http, |m| m.content(result.question));
    }

    let _msg = message.author.direct_message(&context.http, |m| {
        m.content("Answer the above questions individually, and ping me with `done` when you are!")
    });
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

fn parse_pool_activity(
    context: &Context,
    message: &Message,
    message_tokens: &Vec<&str>,
    connection_pool: &Pool,
) {
    use schema::user::dsl::*;
    let connection = connection_pool.get().unwrap();
    let status = get_pool_status(&message.author.id.0, &connection_pool);
    if status > 0 {
        // 2 is creating
        // 1 is joining
        if message_tokens.len() == 0 {
            return;
        }

        if message_tokens[0].to_lowercase() == "done" {
            let updated = diesel::update(user.filter(discord_id.eq(message.author.id.0 as i32)))
                .set(pool_state.eq(0))
                .get_result::<User>(&connection);

            if updated.is_err() {
                let _msg = message
                    .author
                    .direct_message(&context.http, |m| m.content("unknown issue"));
            }

            if status == 2 {
                let _msg = message.author.direct_message(&context.http, |m| {
                    m.content(format!(
                        "Your new pool id is {}!",
                        get_latest_started_pool(&message.author.id.0, &connection_pool)
                    ))
                });
            }

            if status == 1 {
                let _msg = message.author.direct_message(&context.http, |m| {
                    m.content(format!(
                        "Joined pool id {}!",
                        get_latest_joined_pool(&message.author.id.0, &connection_pool)
                    ))
                });
            }

            return;
        }

        if status == 2 {
            insert_question(&message.author.id.0, &connection_pool, &message.content);
        }

        if status == 1 {
            insert_response(&message.author.id.0, &connection_pool, &message.content);
        }

        let _msg = message
            .author
            .direct_message(&context.http, |m| m.content("Add another or say `done`!"));
    };
}

impl EventHandler for Handler {
    fn reaction_add(&self, context: Context, add_reaction: Reaction) {}

    fn message(&self, context: Context, message: Message) {
        let mut data = context.data.write();
        let connection_pool = data.get_mut::<PooledConnection>().unwrap();

        // Sample parsing of message
        let message_tokens: Vec<&str> = message.content.split(" ").collect();
        let message_author_id = message.author.id.0;

        if message_tokens[0] == "!utilbot" || message.is_private() {
            // Keep this first to guarantee that the user exists
            // Test existance of message sender
            if is_user_exist(&message_author_id, connection_pool) {
                println!(
                    "You're already in the database and your ID is {}",
                    message_author_id
                );
            // TODO: Add query here to verify that user has been added
            // Insert new user that sent the message
            } else { println!("Bad command"); }
        }

        if message_tokens[0] == "!utilbot" {
            if message_tokens.len() < 2 {
                println!("You gotta gimme a command first!");
                return;
            }
            if message_tokens[1] == "pool" {
                match message_tokens[2] {
                    "start" => start_pool(&context, &message, &message_tokens, &connection_pool),
                    "join" => join_pool(&context, &message, &message_tokens, &connection_pool),
                    "check" => check_pool(&context, &message, &message_tokens),
                    "skrt" => skrt_pool(&context, &message, &message_tokens),
                    "match" => match_pool(&context, &message, &message_tokens),
                    _ => println!("Bad pool command"),
                }
            } else if message_tokens[1] == "clear" {
                if is_user_exist(&message_author_id, connection_pool) {
                    clear_user_languages(&message_author_id, connection_pool);
                    
                } else { println!("No record present to clear"); }
            } else if message_tokens[1] == "view" {
                if is_user_exist(&message_author_id, connection_pool) {
                    let languages = get_user_languages(&message_author_id, connection_pool);
                    println!("Languages: ");
                    for language in languages { println!("{}", language); }
                } else { println!("No record present to view"); }
                reply_for_add_view(&message_author_id, &message, &context, connection_pool);
            } else if message_tokens[1] == "add" {
                if message_tokens.len() < 3 {
                    println!("Not enough arguments");
                    return;
                }
                let cfg_strings: Vec<&str> = message_tokens[2..].to_vec();
                let mut strings_vec: Vec<String> = Vec::new();
                for s in &cfg_strings { strings_vec.push(s.to_string()); }
                if is_user_exist(&message_author_id, connection_pool) {
                    update_user_languages(&message_author_id, strings_vec, connection_pool);
                } else {
                    insert_user(&message_author_id, strings_vec, connection_pool);
                }
                reply_for_add_view(&message_author_id, &message, &context, connection_pool);
            } else if message_tokens[1] == "get-help" {
                if message_tokens.len() < 4 {
                    println!("Not enough arguments");
                    return;
                }
                let language: String = message_tokens[2].to_string();
                let question: Vec<&str> = message_tokens[3..].to_vec();
                let result_ids: Vec<i64> = get_proficient_users(connection_pool, &language);
                ping_proficient_users(&message_author_id, &result_ids, &language, &question, &message, &context);
                // Ping users here
            } else { println!("Bad command"); }
        }

        if message.is_private() {
            parse_pool_activity(&context, &message, &message_tokens, connection_pool);
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
