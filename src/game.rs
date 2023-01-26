use comfy_table::presets::UTF8_FULL;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::combat::{Attack, GetAll, Ship};
use color_eyre::Result;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, Cell, Color, ContentArrangement};
use sqlx::SqlitePool;

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

    sqlx::query_as!(Fleet, r#"insert into fleets values (?1, "Starfleet")"#, fa)
        .execute(&mut conn)
        .await?;
    sqlx::query_as!(
        Fleet,
        r#"insert into fleets values (?1, "Klingon Imperial Fleet")"#,
        fb
    )
    .execute(&mut conn)
    .await?;

    sqlx::query!(
        r#"insert into ships values (?1, "USS Enterprise", ?2, 10)"#,
        a,
        fa
    )
    .execute(&mut conn)
    .await?;
    sqlx::query!(
        r#"insert into ships values (?1, "Klingon Warbird", ?2, 10)"#,
        b,
        fb
    )
    .execute(&mut conn)
    .await?;

    Ok(())
}
pub async fn run() -> Result<()> {
    let connection = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    lazy_static::lazy_static! {
        static ref COUNTER: AtomicUsize = AtomicUsize::new(0);
    }

    if std::env::var("DB_RESET").is_ok() {
        db_reset(&connection).await?;
    }

    loop {
        println!(
            "Turn {}",
            COUNTER.swap(COUNTER.load(Ordering::Relaxed) + 1, Ordering::Relaxed)
        );
        match turn(&connection).await? {
            Outcome::Complete => return Ok(()),
            Outcome::Continue => (),
        }
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

    let mut table = comfy_table::Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Fleet"),
            Cell::new("Name"),
            Cell::new("Integrity"),
        ]);

    for Ship {
        name,
        fleet,
        integrity,
        ..
    } in Ship::all(connection).await?
    {
        let fleet = sqlx::query!("select (name) from fleets where id = ?1", fleet)
            .fetch_one(connection)
            .await?
            .name;

        table.add_row(vec![
            Cell::new(fleet),
            Cell::new(name),
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
