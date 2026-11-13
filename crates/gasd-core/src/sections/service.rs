use crate::{Result, Section, Service, validation::nonblank};

impl Service {
    pub fn get_sections(&self) -> Result<Vec<Section>> {
        self.repository.get_sections()
    }

    pub fn get_section(&self, id: i64) -> Result<Section> {
        self.repository.get_section(id)
    }

    pub fn create_section(&mut self, name: &str) -> Result<Section> {
        self.repository.create_section(nonblank(name, "name")?)
    }

    pub fn rename_section(&mut self, id: i64, name: &str) -> Result<Section> {
        self.repository.rename_section(id, nonblank(name, "name")?)
    }

    pub fn delete_section(&mut self, id: i64) -> Result<()> {
        self.repository.delete_section(id)
    }

    pub fn reorder_sections(&mut self, ids: &[i64]) -> Result<()> {
        self.repository.reorder_sections(ids)
    }
}
