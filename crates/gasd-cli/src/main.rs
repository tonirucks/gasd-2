use std::{
    error::Error,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Args, Parser, Subcommand};
use gasd_core::{Section, Task, TaskStatus, TaskUpdate};
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
    /// Remove the database file (all data is lost). Close the desktop app first.
    Clean {
        /// Confirm permanent deletion without an interactive prompt
        #[arg(long, required = true)]
        yes: bool,
    },
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
        /// Target section (defaults to the first section in board order)
        #[arg(long)]
        section: Option<i64>,
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
        /// Filter by status: open, inProgress, waiting, done
        #[arg(long, value_parser = parse_status)]
        status: Option<TaskStatus>,
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
#[command(group(clap::ArgGroup::new("changes").required(true).multiple(true).args(["title", "description", "section", "status"])))]
struct TaskEdit {
    id: i64,
    #[arg(long)]
    title: Option<String>,
    /// Set the description (use an empty string to clear it)
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    section: Option<i64>,
    /// Set status: open, inProgress, waiting, done
    #[arg(long, value_parser = parse_status)]
    status: Option<TaskStatus>,
}

fn parse_status(value: &str) -> Result<TaskStatus, String> {
    match value {
        "open" => Ok(TaskStatus::Open),
        "inProgress" => Ok(TaskStatus::InProgress),
        "waiting" => Ok(TaskStatus::Waiting),
        "done" => Ok(TaskStatus::Done),
        _ => Err("expected open, inProgress, waiting, or done".into()),
    }
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
    print_rows(
        rows,
        json,
        "ID\tSECTION\tSTATUS\tTITLE\tDESCRIPTION\tFOLDER",
        |task| {
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                task.id,
                task.section_id,
                match task.status {
                    TaskStatus::Open => "open",
                    TaskStatus::InProgress => "inProgress",
                    TaskStatus::Waiting => "waiting",
                    TaskStatus::Done => "done",
                },
                text(&task.title),
                text(&task.description),
                text(task.folder.as_deref().unwrap_or(""))
            )
        },
    )
}

fn sections(rows: &[Section], json: bool) -> Result<(), Box<dyn Error>> {
    print_rows(rows, json, "ID\tNAME", |section| {
        format!("{}\t{}", section.id, text(&section.name))
    })
}

fn clean(path: &Path, json: bool) -> Result<(), Box<dyn Error>> {
    if path.as_os_str().is_empty() {
        return Err(gasd_core::storage::Error::EmptyPath.into());
    }
    let removed = match std::fs::remove_file(path) {
        Ok(()) => true,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(error) => {
            return Err(format!(
                "could not remove database {}: {error}",
                text(&path.display().to_string())
            )
            .into());
        }
    };
    #[derive(Serialize)]
    struct CleanResult<'a> {
        database: &'a Path,
        removed: bool,
    }
    print_rows(
        &[CleanResult {
            database: path,
            removed,
        }],
        json,
        "DATABASE\tREMOVED",
        |result| {
            format!(
                "{}\t{}",
                text(&result.database.display().to_string()),
                result.removed
            )
        },
    )
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let path = match cli.database {
        Some(path) => path,
        None => gasd_core::storage::database_path()?,
    };
    // Cleaning must work even when an old or corrupt schema cannot be opened.
    if matches!(&cli.command, Command::Clean { .. }) {
        return clean(&path, cli.json);
    }
    let mut service = gasd_core::storage::open(path)?;
    match cli.command {
        Command::Clean { .. } => unreachable!("clean is handled before opening the database"),
        Command::Add {
            item: Add::Task { title, section },
        } => {
            let section = match section {
                Some(id) => id,
                None => service
                    .get_sections()?
                    .first()
                    .ok_or(
                        "no sections available; create one with `gasd add section <name>` first",
                    )?
                    .id,
            };
            let directory = std::env::current_dir()?;
            let folder = directory
                .to_str()
                .ok_or("current directory is not valid Unicode")?;
            tasks(
                &[service.create_task_with_folder(section, &title, folder)?],
                cli.json,
            )
        }
        Command::Add {
            item: Add::Section { name },
        } => sections(&[service.create_section(&name)?], cli.json),
        Command::List {
            item: Some(List::Sections),
        } => sections(&service.get_sections()?, cli.json),
        Command::List { item } => {
            let mut rows = service.get_tasks()?;
            if let Some(List::Tasks { section, status }) = item {
                if let Some(id) = section {
                    service.get_section(id)?;
                }
                rows.retain(|task| {
                    section.is_none_or(|id| task.section_id == id)
                        && status.is_none_or(|status| task.status == status)
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
                    description: edit.description.unwrap_or(old.description),
                    section_id: edit.section.unwrap_or(old.section_id),
                    status: edit.status.unwrap_or(old.status),
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
