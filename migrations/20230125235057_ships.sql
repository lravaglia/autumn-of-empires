-- Add migration script here
create table if not exists ships 
( id text primary key not null
, name text not null
, fleet text not null
, integrity integer   not null
)

