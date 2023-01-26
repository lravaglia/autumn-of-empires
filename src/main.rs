#![feature(no_coverage)]
mod random;

use color_eyre::Result;
use dotenv::dotenv;
use sqlx::prelude::*;

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
struct Ship {
    id: String,
    integrity: i32,
}

#[derive(Debug, Default, PartialEq, Eq, FromRow)]
struct Attack {
    id: String,
    target: String,
}

#[no_coverage]
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();
    dotenv()?;

    let mut inc = 0;

    let connection = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    {
        let mut conn = connection.acquire().await?;

        sqlx::query!("DELETE FROM ships").execute(&mut conn).await?;

        sqlx::query!(r#"INSERT INTO ships VALUES ("One", 1)"#)
            .execute(&mut conn)
            .await?;
        sqlx::query!(r#"INSERT INTO ships VALUES ("Two", 2)"#)
            .execute(&mut conn)
            .await?;
    }

    loop {
        let mut conn = connection.acquire().await?;

        sqlx::query!("DELETE FROM attacks")
            .execute(&mut conn)
            .await?;

        for ship in sqlx::query!("SELECT * FROM ships")
            .fetch_all(&mut conn)
            .await?
        {
            inc += 1;
            sqlx::query!("INSERT INTO attacks VALUES (?1, ?2)", inc, ship.id)
                .execute(&mut conn)
                .await?;
        }

        for attack in sqlx::query!("SELECT * FROM attacks")
            .fetch_all(&mut conn)
            .await?
        {
            let p_ship = sqlx::query!("SELECT (integrity) FROM ships WHERE id = ?1", attack.target)
                .fetch_one(&mut conn)
                .await?;
            let integrity = p_ship.integrity - 1;
            sqlx::query!(
                "UPDATE ships SET integrity = ?1 WHERE id = ?2",
                integrity,
                attack.target
            )
            .execute(&mut conn)
            .await?;
        }

        for ship in sqlx::query!("SELECT * FROM ships")
            .fetch_all(&mut conn)
            .await?
        {
            if ship.integrity < 1 {
                sqlx::query!("DELETE FROM ships WHERE id = ?", ship.id)
                    .execute(&mut conn)
                    .await?;
            }
        }

        for ship in sqlx::query!("SELECT * FROM ships")
            .fetch_all(&mut conn)
            .await?
        {
            println!("{}", ship.integrity);
        }

        break;
    }
    Ok(())
}
