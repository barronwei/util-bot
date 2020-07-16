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

fn get_pool(uid: &i32, connection_pool: &Pool) -> Result<MatchAdmin, ()> {
    use schema::match_admin::dsl::*;
    let connection = connection_pool.get().unwrap();
    let mut results = match_admin
        .filter(id.eq(*uid as i32))
        .load::<MatchAdmin>(&connection)
        .expect("error getting pool");
    if results.len() == 0 {
        Err(())
    } else {
        Ok(results.remove(0))
    }
}

fn deactivate_pool(uid: &i32, connection_pool: &Pool) -> bool {
    use schema::match_admin::dsl::*;
    let connection = connection_pool.get().unwrap();
    let updated = diesel::update(match_admin.filter(id.eq(*uid)))
        .set(status.eq(false))
        .get_result::<MatchAdmin>(&connection);
    if updated.is_err() {
        println!("Failed to deactivate pool {}", *uid);
        return false;
    }
    true
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

fn generate_pool_matches(pool: &MatchAdmin, connection_pool: &Pool) {
    let connection = connection_pool.get().unwrap();
    let results = schema::pool_questions::dsl::pool_questions
        .filter(schema::pool_questions::dsl::pool_id.eq(pool.id))
        .load::<PoolQuestions>(&connection)
        .expect("error getting pool questions");

    let question_count = results.len() as i64;
    let response_headers = schema::match_responses::dsl::match_responses
        .filter(schema::match_responses::dsl::match_id.eq(pool.id))
        .load::<MatchResponses>(&connection)
        .expect("error getting pool response headers");

    let total_pool_size = response_headers.len() as f64;
    let mut responses_by_user: Vec<Vec<PoolResponses>> = Vec::new();
    for response_header in response_headers.into_iter() {
        let responses = schema::pool_responses::dsl::pool_responses
            .filter(schema::pool_responses::dsl::response_id.eq(response_header.id))
            .limit(question_count)
            .load::<PoolResponses>(&connection)
            .expect("error getting pool response from header");
        responses_by_user.push(responses);
    }

    // compute group sizes
    // if not evenly divisble, split what would be the last full group plus the leftover members into two groups
    let group_count_at_max_size =
        ((total_pool_size - pool.group_size as f64) / pool.group_size as f64).floor();
    let leftover_count = total_pool_size - (group_count_at_max_size * pool.group_size as f64);
    let last_group_size = (leftover_count / pool.group_size as f64).floor();
    let second_to_last_group_size = leftover_count - last_group_size;
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

    let poolid = message_tokens[3].parse::<i32>().unwrap();
    let res = get_pool(&poolid, &connection_pool);
    let match_admin;
    match res {
        Err(_) => {
            let _msg = message.author.direct_message(&context.http, |m| {
                m.content(format!("pool_id {} is not valid", poolid))
            });
            return;
        }
        Ok(match_result) => match_admin = match_result,
    }

    if match_admin.status == false {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("pool_id {} is inactive", poolid))
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
        m.content("Answer the above questions individually with `yes` or `no`, and ping me with `done` when you are!")
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

fn match_pool(
    context: &Context,
    message: &Message,
    message_tokens: &Vec<&str>,
    connection_pool: &Pool,
) {
    if message_tokens.len() < 4 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("Please use `!utilbot pool match POOL_ID`!"))
        });
        return;
    }

    if message_tokens[3].parse::<i32>().is_err() || message_tokens[3].parse::<i32>().unwrap() < 1 {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("Please use a proper pool id"))
        });
        return;
    }

    let pool_id = message_tokens[3].parse::<i32>().unwrap();
    let res = get_pool(&pool_id, &connection_pool);
    let match_admin;
    match res {
        Err(_) => {
            let _msg = message.author.direct_message(&context.http, |m| {
                m.content(format!("pool_id {} is not valid", pool_id))
            });
            return;
        }
        Ok(match_result) => match_admin = match_result,
    }

    let discord_id = message.author.id.0;
    if match_admin.user_id != get_user_id(&discord_id, &connection_pool) {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("You do not own pool {}", pool_id))
        });
        return;
    }

    if !match_admin.status {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("pool {} is not active", pool_id))
        });
        return;
    }

    if !deactivate_pool(&pool_id, &connection_pool) {
        let _msg = message.author.direct_message(&context.http, |m| {
            m.content(format!("Failed to deactivate pool {}", pool_id))
        });
        return;
    }

    generate_pool_matches(&match_admin, &connection_pool);
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
            if message_tokens[0].to_lowercase() != "yes" && message_tokens[0].to_lowercase() != "no"
            {
                let _msg = message.author.direct_message(&context.http, |m| {
                    m.content("Please say `yes`, `no`, or `done`!")
                });
                return;
            }
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
            } else {
                insert_user(&message.author.id.0, &message.content, connection_pool);
                println!(
                    "You've been added to the database. Your ID is {}",
                    message_author_id
                )
            }
        }

        if message_tokens[0] == "!utilbot" {
            if message_tokens[1] == "pool" {
                match message_tokens[2] {
                    "start" => start_pool(&context, &message, &message_tokens, &connection_pool),
                    "join" => join_pool(&context, &message, &message_tokens, &connection_pool),
                    "check" => check_pool(&context, &message, &message_tokens),
                    "skrt" => skrt_pool(&context, &message, &message_tokens),
                    "match" => match_pool(&context, &message, &message_tokens, &connection_pool),
                    _ => println!("Bad pool command"),
                }
            }
        }

        if message.is_private() {
            parse_pool_activity(&context, &message, &message_tokens, connection_pool);
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
