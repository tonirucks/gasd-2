use crate::{Result, Service, Task, TaskList, TaskUpdate, validation::nonblank};

impl Service {
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        self.repository.get_tasks()
    }

    pub fn get_task(&self, id: i64) -> Result<Task> {
        self.repository.get_task(id)
    }

    pub fn create_task(&mut self, section_id: i64, title: &str) -> Result<Task> {
        self.repository
            .create_task(section_id, nonblank(title, "title")?)
    }

    pub fn update_task(&mut self, id: i64, mut update: TaskUpdate) -> Result<Task> {
        update.title = nonblank(&update.title, "title")?.to_owned();
        self.repository.update_task(id, &update)
    }

    pub fn delete_task(&mut self, id: i64) -> Result<()> {
        self.repository.delete_task(id)
    }

    pub fn reorder_tasks(&mut self, list: TaskList, ids: &[i64]) -> Result<()> {
        self.repository.reorder_tasks(list, ids)
    }

    /// Moves one task and orders its destination in a single transaction.
    pub fn place_task(&mut self, id: i64, list: TaskList, ids: &[i64]) -> Result<()> {
        self.repository.place_task(id, list, ids)
    }
}
