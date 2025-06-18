-- Add migration script here
CREATE TABLE stops (
    stop_number INTEGER NOT NULL PRIMARY KEY,
    alias TEXT NOT NULL
);

CREATE TABLE bus_list (
    stop INTEGER,
    bus TEXT
);

CREATE TABLE stopcollection (
    collection_name TEXT,
    stop_number INTEGER
);