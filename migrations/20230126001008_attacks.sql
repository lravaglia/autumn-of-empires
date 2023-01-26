-- Add migration script here
create table if not exists attacks
( id integer primary key not null
, target text not null
)
