-- Your SQL goes here
create table if not exists "user" (
    id serial primary key,
    discord_id integer not null,
    languages text not null,
    pool_state integer not null
);

create table if not exists "match_admin" (
    id serial primary key,
    user_id integer not null references "user" (id) on delete cascade,
    status bool not null
);

create table if not exists "pool_questions" (
    id serial primary key,
    pool_id integer not null references "match_admin" (id) on delete cascade,
    question text not null
);

create table if not exists "match_responses" (
    id serial primary key,
    match_id integer not null references "match_admin" (id) on delete cascade,
    user_id integer not null references "user" (id) on delete cascade
);

create table if not exists "pool_responses" (
    id serial primary key,
    response_id integer not null references "match_responses" (id) on delete cascade,
    answer text not null
);

create table if not exists "match_groups" (
    id serial primary key,
    match_id integer not null references "match_admin" (id) on delete cascade,
    members integer[] not null
);

