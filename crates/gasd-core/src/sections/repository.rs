use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::sqlite::{require_changed, validate_order};
use crate::{Error, Result, Section, SqliteRepository};

impl SqliteRepository {
    pub fn get_sections(&self) -> Result<Vec<Section>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, position FROM sections ORDER BY position, id")?;
        Ok(statement
            .query_map([], section_row)?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn get_section(&self, id: i64) -> Result<Section> {
        find_section(&self.connection, id)
    }

    pub fn create_section(&mut self, name: &str) -> Result<Section> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO sections(name, position)
             VALUES (?1, (SELECT coalesce(max(position), -1) + 1 FROM sections))",
            [name],
        )?;
        let section = find_section(&tx, tx.last_insert_rowid())?;
        tx.commit()?;
        Ok(section)
    }

    pub fn rename_section(&mut self, id: i64, name: &str) -> Result<Section> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_changed(
            tx.execute(
                "UPDATE sections SET name = ?1 WHERE id = ?2",
                params![name, id],
            )?,
            "section",
            id,
        )?;
        let section = find_section(&tx, id)?;
        tx.commit()?;
        Ok(section)
    }

    /// Deletes the section and all its tasks, including completed tasks.
    pub fn delete_section(&mut self, id: i64) -> Result<()> {
        require_changed(
            self.connection
                .execute("DELETE FROM sections WHERE id = ?1", [id])?,
            "section",
            id,
        )
    }

    /// Validates complete membership and updates ordering atomically.
    pub fn reorder_sections(&mut self, ids: &[i64]) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = tx
            .prepare("SELECT id FROM sections")?
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<i64>>>()?;
        validate_order(&existing, ids)?;
        for (position, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE sections SET position = ?1 WHERE id = ?2",
                params![position as i64, id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

fn section_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Section> {
    Ok(Section {
        id: row.get(0)?,
        name: row.get(1)?,
        position: row.get(2)?,
    })
}

pub(crate) fn find_section(connection: &Connection, id: i64) -> Result<Section> {
    connection
        .query_row(
            "SELECT id, name, position FROM sections WHERE id = ?1",
            [id],
            section_row,
        )
        .optional()?
        .ok_or(Error::NotFound {
            entity: "section",
            id,
        })
}
