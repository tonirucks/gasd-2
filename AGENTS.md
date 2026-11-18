# Development constraints

- Before making database schema changes, ask the owner whether to modify the
  latest migration or create a new migration. Do not choose automatically.
  Databases are now in active use; preserve existing data and support upgrades
  when adding a new migration.
- Do not reset or delete the database unless explicitly asked.
