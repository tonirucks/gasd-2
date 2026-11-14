use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::sections::repository::find_section;
use crate::sqlite::{require_changed, validate_order};
use crate::{Error, Result, SqliteRepository, Task, TaskList, TaskStatus, TaskUpdate};

impl SqliteRepository {
    /// Returns tasks in section, status column, and task order.
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let mut statement = self.connection.prepare(
            "SELECT t.id, t.title, t.section_id, t.position, t.status, t.status_changed_at
             FROM tasks t JOIN sections s ON s.id = t.section_id
             ORDER BY s.position, s.id,
                      CASE t.status WHEN 'open' THEN 0 WHEN 'inProgress' THEN 1
                           WHEN 'waiting' THEN 2 ELSE 3 END,
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
        let position = next_task_position(&tx, section_id, TaskStatus::Open)?;
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
        let changed_list =
            current.status != update.status || current.section_id != update.section_id;
        let position = if changed_list {
            next_task_position(&tx, update.section_id, update.status)?
        } else {
            current.position
        };
        tx.execute(
            "UPDATE tasks SET title = ?1, section_id = ?2, status = ?3, position = ?4
             WHERE id = ?5",
            params![update.title, update.section_id, update.status, position, id],
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

    /// Status changes append to the destination column.
    pub fn set_task_status(&mut self, id: i64, status: TaskStatus) -> Result<Task> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = find_task(&tx, id)?;
        let position = if current.status != status {
            next_task_position(&tx, current.section_id, status)?
        } else {
            current.position
        };
        tx.execute(
            "UPDATE tasks SET status = ?1, position = ?2 WHERE id = ?3",
            params![status, position, id],
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
        find_task(&tx, id)?;
        find_section(&tx, list.section_id)?;
        // Membership validation below also rejects a destination omitting the moved task.
        tx.execute(
            "UPDATE tasks SET section_id = ?1, status = ?2 WHERE id = ?3",
            params![list.section_id, list.status, id],
        )?;
        reorder_tasks(&tx, list, ids)?;
        tx.commit()?;
        Ok(())
    }
}

fn reorder_tasks(connection: &Connection, list: TaskList, ids: &[i64]) -> Result<()> {
    find_section(connection, list.section_id)?;
    let existing = connection
        .prepare("SELECT id FROM tasks WHERE status = ?1 AND section_id = ?2")?
        .query_map(params![list.status, list.section_id], |row| row.get(0))?
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
        position: row.get(3)?,
        status: row.get(4)?,
        status_changed_at: row.get(5)?,
    })
}

fn find_task(connection: &Connection, id: i64) -> Result<Task> {
    connection
        .query_row(
            "SELECT id, title, section_id, position, status, status_changed_at
             FROM tasks WHERE id = ?1",
            [id],
            task_row,
        )
        .optional()?
        .ok_or(Error::NotFound { entity: "task", id })
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

fn next_task_position(connection: &Connection, section_id: i64, status: TaskStatus) -> Result<i64> {
    Ok(connection.query_row(
        "SELECT coalesce(max(position), -1) + 1 FROM tasks
         WHERE status = ?1 AND section_id = ?2",
        params![status, section_id],
        |row| row.get(0),
    )?)
}
