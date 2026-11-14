use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::sections::repository::find_section;
use crate::sqlite::{require_changed, validate_order};
use crate::{Error, Result, SqliteRepository, Task, TaskList, TaskStatus, TaskUpdate};

impl SqliteRepository {
    /// Returns active tasks in section order, followed by the shared done list.
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let mut statement = self.connection.prepare(
            "SELECT t.id, t.title, t.section_id, t.done, t.position, t.status, t.status_changed_at
             FROM tasks t JOIN sections s ON s.id = t.section_id
             ORDER BY t.done,
                      CASE WHEN t.done = 0 THEN s.position ELSE 0 END,
                      CASE WHEN t.done = 0 THEN s.id ELSE 0 END,
                      t.position, t.id",
        )?;
        Ok(statement
            .query_map([], task_row)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn get_task(&self, id: i64) -> Result<Task> {
        find_task(&self.connection, id)
    }

    pub fn create_task(&mut self, section_id: i64, title: &str) -> Result<Task> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        find_section(&tx, section_id)?;
        let position = next_task_position(&tx, section_id, false)?;
        tx.execute(
            "INSERT INTO tasks(title, section_id, position) VALUES (?1, ?2, ?3)",
            params![title, section_id, position],
        )?;
        let task = find_task(&tx, tx.last_insert_rowid())?;
        tx.commit()?;
        Ok(task)
    }

    /// Moving between lists appends the task to the destination list.
    pub fn update_task(&mut self, id: i64, update: &TaskUpdate) -> Result<Task> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = find_task(&tx, id)?;
        find_section(&tx, update.section_id)?;
        let status = completion_status(current.status, update.done);
        let changed_list = current.done != update.done
            || (!update.done && current.section_id != update.section_id);
        let position = if changed_list {
            next_task_position(&tx, update.section_id, update.done)?
        } else {
            current.position
        };
        tx.execute(
            "UPDATE tasks SET title = ?1, section_id = ?2, done = ?3, position = ?4,
             status = ?6 WHERE id = ?5",
            params![
                update.title,
                update.section_id,
                update.done,
                position,
                id,
                status
            ],
        )?;
        let task = find_task(&tx, id)?;
        tx.commit()?;
        Ok(task)
    }

    pub fn delete_task(&mut self, id: i64) -> Result<()> {
        require_changed(
            self.connection
                .execute("DELETE FROM tasks WHERE id = ?1", [id])?,
            "task",
            id,
        )
    }

    /// Status changes within the active list preserve position. Crossing the
    /// completion boundary appends to the destination list.
    pub fn set_task_status(&mut self, id: i64, status: TaskStatus) -> Result<Task> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = find_task(&tx, id)?;
        let position = if current.done != status.is_done() {
            next_task_position(&tx, current.section_id, status.is_done())?
        } else {
            current.position
        };
        tx.execute(
            "UPDATE tasks SET status = ?1, done = ?2, position = ?3 WHERE id = ?4",
            params![status, status.is_done(), position, id],
        )?;
        let task = find_task(&tx, id)?;
        tx.commit()?;
        Ok(task)
    }

    /// Validates complete membership and updates ordering atomically.
    pub fn reorder_tasks(&mut self, list: TaskList, ids: &[i64]) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        reorder_tasks(&tx, list, ids)?;
        tx.commit()?;
        Ok(())
    }

    pub fn place_task(&mut self, id: i64, list: TaskList, ids: &[i64]) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let task = find_task(&tx, id)?;
        let (section_id, done) = match list {
            TaskList::Active { section_id } => {
                find_section(&tx, section_id)?;
                (section_id, false)
            }
            TaskList::Done => (task.section_id, true),
        };
        // Membership validation below also rejects a destination omitting the moved task.
        tx.execute(
            "UPDATE tasks SET section_id = ?1, done = ?2, status = ?4 WHERE id = ?3",
            params![section_id, done, id, completion_status(task.status, done)],
        )?;
        reorder_tasks(&tx, list, ids)?;
        tx.commit()?;
        Ok(())
    }
}

fn reorder_tasks(connection: &Connection, list: TaskList, ids: &[i64]) -> Result<()> {
    let (section_id, done) = match list {
        TaskList::Active { section_id } => {
            find_section(connection, section_id)?;
            (section_id, false)
        }
        TaskList::Done => (0, true),
    };
    let existing = connection
        .prepare("SELECT id FROM tasks WHERE done = ?1 AND (?1 = 1 OR section_id = ?2)")?
        .query_map(params![done, section_id], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;
    validate_order(&existing, ids)?;
    for (position, id) in ids.iter().enumerate() {
        connection.execute(
            "UPDATE tasks SET position = ?1 WHERE id = ?2",
            params![position as i64, id],
        )?;
    }
    Ok(())
}

fn task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        section_id: row.get(2)?,
        done: row.get(3)?,
        position: row.get(4)?,
        status: row.get(5)?,
        status_changed_at: row.get(6)?,
    })
}

fn find_task(connection: &Connection, id: i64) -> Result<Task> {
    connection
        .query_row(
            "SELECT id, title, section_id, done, position, status, status_changed_at
             FROM tasks WHERE id = ?1",
            [id],
            task_row,
        )
        .optional()?
        .ok_or(Error::NotFound { entity: "task", id })
}

fn completion_status(current: TaskStatus, done: bool) -> TaskStatus {
    if done {
        TaskStatus::Done
    } else if current.is_done() {
        TaskStatus::Open
    } else {
        current
    }
}

impl rusqlite::types::ToSql for TaskStatus {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(match self {
            Self::Open => "open",
            Self::InProgress => "inProgress",
            Self::Waiting => "waiting",
            Self::Done => "done",
        }
        .into())
    }
}

impl rusqlite::types::FromSql for TaskStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_str()? {
            "open" => Ok(Self::Open),
            "inProgress" => Ok(Self::InProgress),
            "waiting" => Ok(Self::Waiting),
            "done" => Ok(Self::Done),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

fn next_task_position(connection: &Connection, section_id: i64, done: bool) -> Result<i64> {
    Ok(connection.query_row(
        "SELECT coalesce(max(position), -1) + 1 FROM tasks
         WHERE done = ?1 AND (?1 = 1 OR section_id = ?2)",
        params![done, section_id],
        |row| row.get(0),
    )?)
}
