use serde_yml::libyml::error;
use sqlx::{error::DatabaseError, Pool, Sqlite};
use PartialEq;

use crate::errors::LabwhereError;

/// LocationType struct
/// A LocationType is a type of location, e.g. Building, Room, etc.
#[derive(Debug, PartialEq, sqlx::FromRow)]
pub struct LocationType {
    /// The unique identifier for the LocationType
    pub id: u32,
    /// The unique name of the LocationType
    pub name: String,
}

/// Implementation of the LocationType struct
impl LocationType {
    /// Create a new LocationType
    /// # Examples
    ///
    /// ```
    /// # #[cfg(doctest)] {
    /// use location_type::LocationType;
    /// let locationType = LocationType::new(1, "Building".to_string());
    /// # }
    /// ```
    pub fn new(id: u32, name: String) -> LocationType {
        LocationType { id, name }
    }

    /// Create a new LocationType
    /// # Examples
    /// ```
    /// # #[cfg(doctest)] {
    /// use location_type::LocationType;
    /// let locationType = LocationType::create("Building".to_string()).await.unwrap();
    /// # }
    /// ```
    pub async fn create(
        name: String,
        connection: &Pool<Sqlite>,
    ) -> Result<LocationType, LabwhereError> {
        return match sqlx::query("INSERT INTO location_types (name) VALUES (?)")
            .bind(name.clone())
            .execute(connection)
            .await
        {
            Ok(insert_query_result) => {
                let id = insert_query_result.last_insert_rowid();
                Ok(LocationType::new(id as u32, name))
            }
            Err(error) => {
                if error.as_database_error().unwrap().is_unique_violation() {
                    Err(LabwhereError::unique_constraint_violation())
                } else {
                    Err(LabwhereError::database_error())
                }
            }
        };
    }

    /// Finds a `LocationType` by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - A string that holds the name of the `LocationType`.
    /// * `connection` - A reference to the SQLite connection pool.
    ///
    /// # Returns
    ///
    /// * `Result<LocationType, LabwhereError>` - On success, returns the `LocationType` with the specified name. On failure, returns a `LabwhereError`.
    ///
    /// # Errors
    ///
    /// This function will return a `LabwhereError` if:
    /// * The `LocationType` with the specified name is not found.
    /// * There is an error executing the query.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(doctest)] {
    /// let location_type = LocationType::find_by_name("Freezer".to_string(), &connection).await.unwrap();
    /// println!("Found LocationType: {:?}", location_type);
    /// # }
    /// ```
    pub async fn find_by_name(
        name: String,
        connection: &Pool<Sqlite>,
    ) -> Result<LocationType, LabwhereError> {
        match sqlx::query_as::<_, LocationType>("SELECT * FROM location_types WHERE name = ?")
            .bind(name)
            .fetch_optional(connection)
            .await
        {
            Ok(location_type) => Ok(location_type.unwrap()),
            Err(_) => Err(LabwhereError::not_found_error("LocationType")),
        }
    }
}

impl Default for LocationType {
    fn default() -> LocationType {
        LocationType {
            id: 1,
            name: "Building".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::initiate_pool;
    use crate::models::location_type::LocationType;

    #[test]
    fn test_location_type_new() {
        let location_type = LocationType::new(1, "Building".to_string());
        assert_eq!(location_type.id, 1);
        assert_eq!(location_type.name, "Building");
    }

    #[tokio::test]
    async fn test_create_location_type() {
        let conn = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &conn)
            .await
            .unwrap();
        assert_eq!(location_type.id, 1);
        assert_eq!(location_type.name, "Freezer");
    }

    #[tokio::test]
    async fn test_find_location_type_by_name() {
        let conn = initiate_pool("sqlite::memory:").await.unwrap();
        let location_type = LocationType::create("Freezer".to_string(), &conn)
            .await
            .unwrap();
        let location_type = LocationType::find_by_name("Freezer".to_string(), &conn)
            .await
            .unwrap();
        assert_eq!(location_type.name, "Freezer");
    }
}
