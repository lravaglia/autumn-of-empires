-- Add migration script here
create table if not exists fleets
( id text primary key not null
, name text not null
)
