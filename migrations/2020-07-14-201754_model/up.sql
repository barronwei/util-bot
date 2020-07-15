-- Your SQL goes here
create table if not exists "user" (
    id serial primary key,
    discord_id integer not null,
    languages text not null
);