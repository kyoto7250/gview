mod app;
mod components;
mod repository;
use std::{
    io::{self, stdout},
    panic,
};

use clap::Parser;

#[derive(Parser)]
#[command(name = "gview")]
#[command(about = "A TUI Viewer for Specific Git Commit IDs")]
struct Args {
    /// Optional commit ID to start from
    #[arg(short, long)]
    commit: Option<String>,
}

use app::Tui;
use color_eyre::{
    config::{EyreHook, HookBuilder, PanicHook},
    eyre,
};
use crossterm::ExecutableCommand;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    terminal::Terminal,
};

pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
    install_panic_hook(panic_hook);
    install_error_hook(eyre_hook)?;
    Ok(())
}

/// Install a panic hook that restores the terminal before printing the panic.
fn install_panic_hook(panic_hook: PanicHook) {
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        panic_hook(panic_info);
    }));
}

/// Install an error hook that restores the terminal before printing the error.
fn install_error_hook(eyre_hook: EyreHook) -> color_eyre::Result<()> {
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(move |error| {
        let _ = restore_terminal();
        eyre_hook(error)
    }))?;
    Ok(())
}

/// Initialize the terminal and enter alternate screen mode.
pub fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

/// Restore the terminal to its original state.
pub fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    let repository_info = repository::RepositoryInfo::new();
    if repository_info.is_err() {
        return Ok(());
    }

    let mut repo_info = repository_info.unwrap();

    // If a commit ID is provided, try to set it
    if let Some(commit_id) = args.commit {
        if repo_info.set_commit_by_id(&commit_id).is_err() {
            eprintln!("Commit not found: {}", commit_id);
            return Ok(());
        }
    }

    install_hooks()?;
    let mut terminal = init_terminal()?;
    let mut app = app::App::new(repo_info);
    app.run(&mut terminal)?;
    restore_terminal()?;
    Ok(())
}
