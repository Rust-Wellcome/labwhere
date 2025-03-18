use crate::errors::LabwhereError;
use crate::models::location_type::LocationType;
use sqlx::{Pool, Sqlite};

pub async fn seed(connection: &Pool<Sqlite>) {
    seed_location_types(connection).await.unwrap()
}

async fn seed_location_types(connection: &Pool<Sqlite>) -> Result<(), LabwhereError> {
    LocationType::create(String::from("freezer"), connection).await?;
    LocationType::create(String::from("box"), connection).await?;

    Ok(())
}
