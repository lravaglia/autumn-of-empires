-- Add migration script here
create table if not exists ships 
( id text primary key not null
, integrity integer   not null
)

