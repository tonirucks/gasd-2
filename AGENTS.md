# Development constraints

- Until the first version is finished, keep all database schema changes in
  `crates/gasd-core/migrations/001_initial.sql` and the schema version at 1.
  Do not add incremental migrations or development-database upgrade paths.
  The owner resets/deletes the database when needed; do not delete it yourself
  unless explicitly asked.
