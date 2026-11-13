use std::{error::Error, path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand};
use gasd_core::{Section, Task, TaskUpdate};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "gasd",
    version,
    about = "Manage tasks and sections without prompts"
)]
struct Cli {
    /// Override the shared database path (otherwise GASD_DATABASE or local app data)
    #[arg(long, global = true)]
    database: Option<PathBuf>,
    /// Emit JSON instead of human-readable output
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a task or section
    Add {
        #[command(subcommand)]
        item: Add,
    },
    /// List tasks (default) or sections
    List {
        #[command(subcommand)]
        item: Option<List>,
    },
    /// Edit a task or rename a section
    Update {
        #[command(subcommand)]
        item: Update,
    },
}

#[derive(Subcommand)]
enum Add {
    Task {
        title: String,
        #[arg(long)]
        section: i64,
    },
    Section {
        name: String,
    },
}

#[derive(Subcommand)]
enum List {
    Tasks {
        #[arg(long)]
        section: Option<i64>,
        /// Only completed tasks
        #[arg(long, conflicts_with = "active")]
        done: bool,
        /// Only incomplete tasks
        #[arg(long)]
        active: bool,
    },
    Sections,
}

#[derive(Subcommand)]
enum Update {
    Task(TaskEdit),
    Section {
        id: i64,
        #[arg(long)]
        name: String,
    },
}

#[derive(Args)]
#[command(group(clap::ArgGroup::new("changes").required(true).multiple(true).args(["title", "section", "done", "active"])))]
struct TaskEdit {
    id: i64,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    section: Option<i64>,
    /// Mark complete (not a toggle)
    #[arg(long, conflicts_with = "active")]
    done: bool,
    /// Reopen the task
    #[arg(long)]
    active: bool,
}

fn print_rows<T: Serialize>(
    rows: &[T],
    json: bool,
    header: &str,
    row: impl Fn(&T) -> String,
) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    if json {
        serde_json::to_writer_pretty(&mut out, rows)?;
        writeln!(out)?;
    } else {
        writeln!(out, "{header}")?;
        for item in rows {
            writeln!(out, "{}", row(item))?;
        }
    }
    Ok(())
}

// Escape control characters so titles cannot inject extra rows or terminal commands.
fn text(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}

fn tasks(rows: &[Task], json: bool) -> Result<(), Box<dyn Error>> {
    print_rows(rows, json, "ID\tSECTION\tSTATE\tTITLE", |task| {
        format!(
            "{}\t{}\t{}\t{}",
            task.id,
            task.section_id,
            if task.done { "done" } else { "active" },
            text(&task.title)
        )
    })
}

fn sections(rows: &[Section], json: bool) -> Result<(), Box<dyn Error>> {
    print_rows(rows, json, "ID\tNAME", |section| {
        format!("{}\t{}", section.id, text(&section.name))
    })
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let path = match cli.database {
        Some(path) => path,
        None => gasd_core::storage::database_path()?,
    };
    let mut service = gasd_core::storage::open(path)?;
    match cli.command {
        Command::Add {
            item: Add::Task { title, section },
        } => tasks(&[service.create_task(section, &title)?], cli.json),
        Command::Add {
            item: Add::Section { name },
        } => sections(&[service.create_section(&name)?], cli.json),
        Command::List {
            item: Some(List::Sections),
        } => sections(&service.get_sections()?, cli.json),
        Command::List { item } => {
            let mut rows = service.get_tasks()?;
            if let Some(List::Tasks {
                section,
                done,
                active,
            }) = item
            {
                if let Some(id) = section {
                    service.get_section(id)?;
                }
                rows.retain(|task| {
                    section.is_none_or(|id| task.section_id == id)
                        && (!done || task.done)
                        && (!active || !task.done)
                });
            }
            tasks(&rows, cli.json)
        }
        Command::Update {
            item: Update::Section { id, name },
        } => sections(&[service.rename_section(id, &name)?], cli.json),
        Command::Update {
            item: Update::Task(edit),
        } => {
            let old = service.get_task(edit.id)?;
            let updated = service.update_task(
                edit.id,
                TaskUpdate {
                    title: edit.title.unwrap_or(old.title),
                    section_id: edit.section.unwrap_or(old.section_id),
                    done: if edit.done {
                        true
                    } else if edit.active {
                        false
                    } else {
                        old.done
                    },
                },
            )?;
            tasks(&[updated], cli.json)
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gasd: {error}");
            ExitCode::FAILURE
        }
    }
}
