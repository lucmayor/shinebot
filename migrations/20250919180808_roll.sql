-- Add migration script here
CREATE TABLE rolls (
    id INTEGER NOT NULL,
    rolls INTEGER,
    dubs_counter INTEGER DEFAULT 0,
    trips_counter INTEGER DEFAULT 0,
    quads_counter INTEGER DEFAULT 0,
    above_counter INTEGER DEFAULT 0,
    elo FLOAT DEFAULT 1600.0
);