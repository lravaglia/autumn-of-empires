use async_trait::async_trait;
use color_eyre::Result;
use sqlx::{prelude::*, SqlitePool};

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
        let r = sqlx::query_as!(Self, "select * from ships")
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
        let r = sqlx::query_as!(Self, "select * from attacks")
            .fetch_all(&mut conn)
            .await?;
        Ok(r)
    }
}

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

    sqlx::query!(r#"insert into ships values (?1, 1)"#, a)
        .execute(&mut conn)
        .await?;
    sqlx::query!(r#"insert into ships values (?1, 2)"#, b)
        .execute(&mut conn)
        .await?;

    Ok(())
}
pub async fn run() -> Result<()> {
    let connection = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    if std::env::var("DB_RESET").is_ok() {
        db_reset(&connection).await?;
    }

    loop {
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

    if Ship::all(connection).await?.len() < 2 {
        return Ok(Outcome::Complete);
    }

    Ok(Outcome::Continue)
}
