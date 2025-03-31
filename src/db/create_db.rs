use log::info;
use sqlx::migrate::MigrateDatabase;

/// Creates an SQLite database.
///
/// # Arguments
///
/// * `path` - The folder where the database will be created.
/// * `environment` - The environment to create the database in e.g. test, dev, prod.
///
/// # Returns
/// Returns a `Result` containing `()` or a `sqlx::Error`.
///
/// Will create a database in the folder with the environment name
///
/// # Examples
/// ```
/// # #[cfg(doctest)] {
/// create_db("src/db", "test").await;
/// }
/// ```
pub async fn create_db(path: Option<String>, environment: &str) -> Result<String, sqlx::Error> {
    let url = match path {
        Some(path) => {
            format!("sqlite://{}/{}.db", path, environment)
        }
        None => {
            format!("sqlite://{}.db", environment)
        }
    };
    info!("Creating the database in {}", url);
    sqlx::Sqlite::create_database(&url).await?;
    Ok(url)
}

#[cfg(test)]
mod tests {
    use crate::db::create_db::create_db;
    // use crate::db::init_db;
    use crate::db::initiate_pool;
    use sqlx::migrate::MigrateDatabase;
    use tokio::fs;

    #[tokio::test]
    async fn test_create_db() {
        let result = create_db(None, "test").await;
        initiate_pool("sqlite://test.db").await.unwrap();
        assert_eq!(result.is_ok(), true);
        sqlx::Sqlite::drop_database("sqlite://test.db")
            .await
            .unwrap();
        fs::remove_file("test.db-wal").await.ok();
        fs::remove_file("test.db-shm").await.ok();
    }

    #[tokio::test]
    async fn test_create_db_with_path() {
        let result = create_db(Some("db".to_string()), "test").await;
        initiate_pool("sqlite://db/test.db").await.unwrap();
        assert_eq!(result.is_ok(), true);
        sqlx::Sqlite::drop_database("sqlite://db/test.db")
            .await
            .unwrap();
        fs::remove_file("db/test.db-wal").await.ok();
        fs::remove_file("db/test.db-shm").await.ok();
    }
}
