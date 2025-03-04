
-- Insert location types
INSERT INTO location_types (name) VALUES ('freezer');
INSERT INTO location_types (name) VALUES ('box');

-- Insert locations
INSERT INTO locations (name, barcode, location_type_id) VALUES ('Freezer 1', 'freezer-1', 1);
INSERT INTO locations (name, barcode, location_type_id) VALUES ('Box 1', 'box-1', 2);

-- Insert labwares
INSERT INTO labwares (barcode, location_id) VALUES ('labware-1', 1);
INSERT INTO labwares (barcode, location_id) VALUES ('labware-2', 1);
INSERT INTO labwares (barcode, location_id) VALUES ('labware-3', 2);
INSERT INTO labwares (barcode, location_id) VALUES ('labware-4', 2);