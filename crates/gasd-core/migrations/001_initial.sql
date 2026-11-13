CREATE TABLE sections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    position INTEGER NOT NULL CHECK (position >= 0)
);

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    done INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1)),
    position INTEGER NOT NULL CHECK (position >= 0)
);

CREATE INDEX sections_order ON sections(position, id);
CREATE INDEX tasks_section_order ON tasks(section_id, done, position, id);
CREATE INDEX tasks_done_order ON tasks(done, position, id);
