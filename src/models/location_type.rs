
/// LocationType struct  
/// A LocationType is a type of location, e.g. Building, Room, etc.
struct LocationType {
  /// The unique identifier for the LocationType
  id: u32,
  /// The unique name of the LocationType
  name: String,
}

/// Implementation of the LocationType struct
impl LocationType {
  /// Create a new LocationType
  /// # Examples
  ///
  /// ```
  /// use location_type::LocationType;
  /// let locationType = LocationType::new(1, "Building".to_string());
  /// ```
  fn new(id: u32, name: String) -> LocationType {
    LocationType { id, name }
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_location_type_new() {
    let location_type = LocationType::new(1, "Building".to_string());
    assert_eq!(location_type.id, 1);
    assert_eq!(location_type.name, "Building");
  }
}
