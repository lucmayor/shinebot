-- Add migration script here
CREATE TABLE stats (
    entry_id INTEGER PRIMARY KEY,
    ts TEXT NOT NULL,
    fails BOOLEAN
);

CREATE TABLE tasks (
    taskid INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    task_desc TEXT NOT NULL,
    time_stamp INTEGER NOT NULL,
    type TEXT NOT NULL,
    status TEXT NOT NULL
);