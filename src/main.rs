use eyre::Result;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::types::Decimal;
use tracing::info;
use serde_json::json;

async fn setup_items_table(pool: &MySqlPool) -> Result<()> {
    let table: Option<(String,)> = sqlx::query_as("SHOW TABLES LIKE 'items'")
        .fetch_optional(pool)
        .await?;

    info!("{:?}", table);

    if table.is_none() {
        let result = sqlx::query(
            r#"CREATE TABLE items (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(64) CHARACTER SET utf8mb4 COLLATE utf8mb4_bin,
            price DECIMAL(8, 2)
        )"#,
        )
        .execute(pool)
        .await?;
        info!("{:?}", result);
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(id) FROM items")
        .fetch_one(pool)
        .await?;

    info!("{:?}", count);

    match count.0 {
        0 => {
            let data: Vec<serde_json::Value> = (0..100)
                .map(|i| json!({
                    "name": format!("item{:04}", i + 1),
                    "price": if i % 2 == 0 { Some(11.25) } else { None },
                }))
                .collect();
            let result = sqlx::query(
                r#"
                INSERT INTO
                    items (name, price)
                SELECT
                    name,
                    price
                FROM
                    JSON_TABLE(?, '$[*]' COLUMNS (
                        name VARCHAR(64) PATH '$.name',
                        price DECIMAL(8, 2) PATH '$.price'
                    )) AS data"#,
            )
            .bind(serde_json::to_string(&data).unwrap())
            .execute(pool)
            .await?;
            info!("{:?}", result);
        }
        100 => {
            info!("Test data has already been set up");
        }
        _ => panic!("`items` table is corrupt. please drop it manually."),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect(std::env!("DATABASE_URL"))
        .await?;

    setup_items_table(&pool).await?;

    let ids = vec![10_i64, 20, 381, 35];

    let rows0 =
        sqlx::query_as::<_, (i64, String, Option<Decimal>)>("SELECT id, name, price FROM items WHERE id IN (?, ?, ?, ?)")
            .bind(ids[0])
            .bind(ids[1])
            .bind(ids[2])
            .bind(ids[3])
            .fetch_all(&pool)
            .await?;
    info!("{:?}", rows0);

    let rows1 = sqlx::query_as::<_, (i64, String, Option<Decimal>)>(
        r#"SELECT
            items.id,
            items.name,
            items.price
        FROM
            items,
            JSON_TABLE(?, '$[*]' COLUMNS( id BIGINT PATH '$' ERROR ON ERROR )) AS ids
        WHERE
            items.id = ids.id"#,
    )
    .bind(format!("{:?}", ids))
    .fetch_all(&pool)
    .await?;
    info!("{:?}", rows1);

    assert_eq!(rows0, rows1);

    Ok(())
}
