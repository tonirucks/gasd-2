ALTER TABLE tasks ADD COLUMN description TEXT NOT NULL DEFAULT '';

CREATE TABLE subtasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1))
);
CREATE INDEX subtasks_task_order ON subtasks(task_id, id);

ALTER TABLE tasks ADD COLUMN folder TEXT CHECK (folder IS NULL OR length(folder) > 0);
