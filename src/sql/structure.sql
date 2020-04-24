create extension if not exists "uuid-ossp";

create table if not exists site(
    id uuid primary key default uuid_generate_v4 (),
    channel_link text not null unique,
    channel_title text not null,
    channel_description text,
    channel_image text,
    items text not null,
    item_title text not null,
    item_link text not null,
    item_description text not null,
    item_pubdate text not null,
    item_guid text
);
