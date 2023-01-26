#![feature(no_coverage)]
mod random;

use async_trait::async_trait;
use color_eyre::Result;
use dotenv::dotenv;
use sqlx::prelude::*;

#[async_trait]
trait GetAll: Sized {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>>;
}

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
struct Ship {
    id: String,
    integrity: i64,
}

#[async_trait]
impl GetAll for Ship {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let mut conn = connection.acquire().await?;
        let r = sqlx::query_as!(Self, "SELECT * FROM ships")
            .fetch_all(&mut conn)
            .await?;
        Ok(r)
    }
}

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
struct Attack {
    id: i64,
    target: String,
}

#[async_trait]
impl GetAll for Attack {
    async fn all(connection: &sqlx::SqlitePool) -> Result<Vec<Self>> {
        let mut conn = connection.acquire().await?;
        let r = sqlx::query_as!(Self, "SELECT * FROM attacks")
            .fetch_all(&mut conn)
            .await?;
        Ok(r)
    }
}

async fn db_reset(connection: &sqlx::SqlitePool) -> Result<()> {
    let mut conn = connection.acquire().await?;

    sqlx::query!("DELETE FROM ships").execute(&mut conn).await?;

    sqlx::query!(r#"INSERT INTO ships VALUES ("One", 1)"#)
        .execute(&mut conn)
        .await?;
    sqlx::query!(r#"INSERT INTO ships VALUES ("Two", 2)"#)
        .execute(&mut conn)
        .await?;

    Ok(())
}

#[no_coverage]
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    dotenv()?;

    let connection = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    if std::env::var("DB_RESET").is_ok() {
        db_reset(&connection).await?;
    }

    loop {
        let mut conn = connection.acquire().await?;
        let mut inc = 0;

        sqlx::query!("DELETE FROM attacks")
            .execute(&mut conn)
            .await?;

        for ship in Ship::all(&connection).await? {
            inc += 1;
            sqlx::query!("INSERT INTO attacks VALUES (?1, ?2)", inc, ship.id)
                .execute(&mut conn)
                .await?;
        }

        for attack in sqlx::query_as!(Attack, "SELECT * FROM attacks")
            .fetch_all(&mut conn)
            .await?
        {
            let ship = sqlx::query_as!(Ship, "SELECT * FROM ships WHERE id = ?1", attack.target)
                .fetch_one(&mut conn)
                .await?;
            let integrity = ship.integrity - 1;
            sqlx::query!(
                "UPDATE ships SET integrity = ?1 WHERE id = ?2",
                integrity,
                ship.id,
            )
            .execute(&mut conn)
            .await?;
        }

        for ship in Ship::all(&connection).await? {
            if ship.integrity < 1 {
                sqlx::query!("DELETE FROM ships WHERE id = ?", ship.id)
                    .execute(&mut conn)
                    .await?;
            }
        }

        let ships = Ship::all(&connection).await?;

        for ship in &ships {
            println!("{}: {}", ship.id, ship.integrity);
        }

        if ships.len() < 2 {
            break;
        }
    }

    Ok(())
}
