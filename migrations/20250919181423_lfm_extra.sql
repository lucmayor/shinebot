-- Add migration script here
CREATE TABLE demonstrational (
    id INTEGER NOT NULL,
    track TEXT NOT NULL,
    artist TEXT NOT NULL,
    album TEXT,
    timestamp INTEGER NOT NULL
);

CREATE TABLE artistscrobs (
    id INTEGER NOT NULL,
    artist TEXT NOT NULL,
    plays INT NOT NULL
);