use std::os::unix::process;

use log::info;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite};

#[derive(Debug, sqlx::FromRow)]
struct Property {
    pub name: String,
    pub value: String,
}

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

/// Seeds the database with initial data.
///
/// This function checks if the database has already been seeded by querying the `properties` table
/// for a property with the name "seeded". If the property exists and its value is `true`, the function
/// will read the `seeds.sql` file and execute the SQL script to seed the database with initial data.
/// If the property does not exist or its value is not `true`, the function will log a message and return.
///
/// # Arguments
///
/// * `connection` - A reference to the SQLite connection pool.
///
/// # Returns
///
/// Returns a `Result` containing `()` or a `sqlx::Error` if an error occurs.
///
/// # Panics
///
/// This function will panic if:
/// * The `seeds.sql` file cannot be read.
/// * There is an error in querying the database.
///
/// # Examples
///
/// ```rust
/// # #[cfg(doctest)] {
/// let conn = initiate_pool("sqlite::memory:").await.unwrap();
/// seed_data(&conn).await.unwrap();
/// # }
/// ```
pub async fn seed_data(connection: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let name: String = "seeded".to_string();
    match sqlx::query_as::<_, Property>("SELECT name, value FROM properties WHERE name = ?")
        .bind(name)
        .fetch_one(connection)
        .await
    {
        Ok(property) => {
            // Check if property.value exists and true.
            // If it is, do not seed.
            // Else, seed.
            if property.value.parse().unwrap() {
                let seeds_sql = std::fs::read_to_string("src/db/seeds.sql").unwrap();
                sqlx::query(&seeds_sql).execute(connection).await?;
                return Ok(());
            } else {
                info!("Not seeding the database.");
                return Ok(());
            }
        }
        Err(err) => {
            // Send an error.
            panic!("Error in seeding data: {:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::create_db::create_db;
    use crate::db::init_db;
    use sqlx::migrate::MigrateDatabase;

    #[tokio::test]
    async fn test_create_db() {
        let result = create_db(None, "test").await;
        init_db("sqlite://test.db").await.unwrap();
        assert_eq!(result.is_ok(), true);
        sqlx::Sqlite::drop_database("sqlite://test.db")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_db_with_path() {
        let result = create_db(Some("src/db".to_string()), "test").await;
        init_db("sqlite://src/db/test.db").await.unwrap();
        assert_eq!(result.is_ok(), true);
        sqlx::Sqlite::drop_database("sqlite://src/db/test.db")
            .await
            .unwrap();
    }
}
