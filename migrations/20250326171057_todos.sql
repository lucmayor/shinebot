-- Add migration script here
CREATE TABLE tasks (
    taskid INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    task_desc TEXT NOT NULL,
    time_stamp INTEGER NOT NULL
);