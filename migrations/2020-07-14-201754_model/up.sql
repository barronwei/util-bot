-- Your SQL goes here
create table if not exists "user" (
    id serial primary key,
    discord_id integer not null,
    languages text[] not null,
    group_state integer not null
);

create table if not exists "match_admin" (
    id serial primary key,
    questions text[] not null,
    user_id integer not null references "user" (id) on delete cascade
);

create table if not exists "match_responses" (
    id serial primary key,
    answers bool[] not null,
    match_id integer not null references "match_admin" (id) on delete cascade,
    user_id integer not null references "user" (id) on delete cascade
);

create table if not exists "match_groups" (
    id serial primary key,
    members integer[] not null,
    match_id integer not null references "match_admin" (id) on delete cascade
);

