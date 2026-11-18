use crate::{validation::nonblank, Result, Service, Task, TaskList, TaskStatus, TaskUpdate};

impl Service {
    pub fn create_subtask(&mut self, task_id: i64, title: &str) -> Result<Task> {
        self.repository
            .create_subtask(task_id, nonblank(title, "title")?)
    }

    pub fn update_subtask(
        &mut self,
        task_id: i64,
        id: i64,
        title: &str,
        completed: bool,
    ) -> Result<Task> {
        self.repository
            .update_subtask(task_id, id, nonblank(title, "title")?, completed)
    }

    pub fn delete_subtask(&mut self, task_id: i64, id: i64) -> Result<Task> {
        self.repository.delete_subtask(task_id, id)
    }

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

    pub fn set_task_status(&mut self, id: i64, status: TaskStatus) -> Result<Task> {
        self.repository.set_task_status(id, status)
    }

    pub fn reorder_tasks(&mut self, list: TaskList, ids: &[i64]) -> Result<()> {
        self.repository.reorder_tasks(list, ids)
    }

    /// Moves one task and orders its destination in a single transaction.
    pub fn place_task(&mut self, id: i64, list: TaskList, ids: &[i64]) -> Result<()> {
        self.repository.place_task(id, list, ids)
    }
}
