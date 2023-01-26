use comfy_table::presets::UTF8_FULL;
use std::{cell::Cell, sync::Mutex};

use crate::combat::{Attack, GetAll, Ship};
use color_eyre::Result;
use sqlx::{prelude::*, SqlitePool};

/// # Id
/// Generates a new version 7 uuid.
fn id() -> String {
    use uuid::{NoContext, Timestamp, Uuid};
    Uuid::new_v7(Timestamp::now(NoContext)).to_string()
}

async fn db_reset(connection: &sqlx::SqlitePool) -> Result<()> {
    let mut conn = connection.acquire().await?;

    sqlx::query!("delete from ships").execute(&mut conn).await?;

    let (a, b) = (id(), id());
    let (fa, fb) = (id(), id());

    sqlx::query!(r#"insert into ships values (?1, ?2, 10)"#, a, fa)
        .execute(&mut conn)
        .await?;
    sqlx::query!(r#"insert into ships values (?1, ?2, 10)"#, b, fb)
        .execute(&mut conn)
        .await?;

    Ok(())
}
pub async fn run() -> Result<()> {
    let connection = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    lazy_static::lazy_static! {
        static ref COUNTER: Mutex<u64> = Mutex::new(0);
    }
    let mut counter = COUNTER.lock().unwrap();

    if std::env::var("DB_RESET").is_ok() {
        db_reset(&connection).await?;
    }

    loop {
        println!("Turn {}", counter);
        match turn(&connection).await? {
            Outcome::Complete => return Ok(()),
            Outcome::Continue => (),
        }
        *counter += 1;
    }
}

pub enum Outcome {
    Continue,
    Complete,
}

pub async fn turn(connection: &SqlitePool) -> Result<Outcome> {
    let mut conn = connection.acquire().await?;
    let mut inc = 0;

    sqlx::query!("delete from attacks")
        .execute(&mut conn)
        .await?;

    for ship in Ship::all(connection).await? {
        inc += 1;
        sqlx::query!("insert into attacks values (?1, ?2)", inc, ship.id)
            .execute(&mut conn)
            .await?;
    }

    for attack in Attack::all(connection).await? {
        let ship = sqlx::query_as!(Ship, "select * from ships where id = ?1", attack.target)
            .fetch_one(&mut conn)
            .await?;
        let integrity = ship.integrity - 1;
        sqlx::query!(
            "update ships set integrity = ?1 where id = ?2",
            integrity,
            ship.id,
        )
        .execute(&mut conn)
        .await?;
    }

    for ship in Ship::all(connection).await? {
        if ship.integrity < 1 {
            sqlx::query!("delete from ships where id = ?", ship.id)
                .execute(&mut conn)
                .await?;
        }
    }

    use comfy_table::{modifiers::*, *};
    let mut table = comfy_table::Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Fleet"),
            Cell::new("Identity"),
            Cell::new("Integrity"),
        ]);

    for Ship {
        id,
        fleet,
        integrity,
    } in Ship::all(connection).await?
    {
        table.add_row(vec![
            Cell::new(id),
            Cell::new(fleet),
            match integrity {
                0..4 => Cell::new(integrity).fg(Color::Red),
                _ => Cell::new(integrity).fg(Color::Yellow),
            },
        ]);
    }
    println!("{table}");

    if Ship::all(connection).await?.len() < 2 {
        return Ok(Outcome::Complete);
    }

    Ok(Outcome::Continue)
}
