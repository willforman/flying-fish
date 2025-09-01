use std::{
    env,
    fs::File,
    io::{self, BufRead},
    panic,
    path::PathBuf,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use engine::{
    perft, search, HyperbolaQuintessenceMoveGen, Position, SearchParams,
    HYPERBOLA_QUINTESSENCE_MOVE_GEN, POSITION_EVALUATOR,
};
use mimalloc::MiMalloc;
use tracing::{debug, level_filters::LevelFilter, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, prelude::*, util::SubscriberInitExt, Registry};

use cli::{LOGS_DIRECTORY, UCI};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

static MOVE_GEN: HyperbolaQuintessenceMoveGen = HYPERBOLA_QUINTESSENCE_MOVE_GEN;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Search a position for the best move.
    Search {
        fen: String,
        depth: u64,
    },
    Perft {
        fen: String,
        depth: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    enable_logging()?;

    match cli.command {
        Some(Commands::Search { fen, depth }) => cli_search(&fen, depth),
        Some(Commands::Perft { fen, depth }) => cli_perft(&fen, depth),
        None => uci_main_loop(),
    }
}

fn cli_search(fen: &str, depth: u64) -> Result<()> {
    let position = Position::from_fen(fen)?;
    let search_params = SearchParams {
        max_depth: Some(depth),
        ..Default::default()
    };
    let (best_move, _) = search(
        &position,
        &search_params,
        MOVE_GEN,
        POSITION_EVALUATOR,
        Arc::new(AtomicBool::new(false)),
    )?;
    println!(
        "{:?}",
        best_move
            .expect("Should have found a move.")
            .to_string()
            .to_lowercase()
    );
    Ok(())
}

fn cli_perft(fen: &str, depth: u64) -> Result<()> {
    let position = Position::from_fen(fen)?;
    let (_, tot_moves) = perft(&position, depth as usize, HYPERBOLA_QUINTESSENCE_MOVE_GEN);
    println!("{:?} moves", tot_moves);
    Ok(())
}

fn uci_main_loop() -> Result<()> {
    let mut uci = UCI::new(MOVE_GEN);

    for line in io::stdin().lock().lines().map(|r| r.unwrap()) {
        debug!("{}", line);
        let cmd_res = uci.handle_command(&line);

        if let Err(err) = cmd_res {
            warn!("{}", err);
        }
    }
    Ok(())
}

fn enable_logging() -> Result<()> {
    let log_path = if let Ok(log_path_str) = env::var("FLYING_FISH_LOG_PATH") {
        PathBuf::from_str(&log_path_str)?
    } else {
        get_default_log_path()?
    };

    let log_file =
        File::create(log_path.clone()).context(format!("Couldn't create file {:?}", log_path))?;

    // UCI message specific targets:
    // - `uci`: this crate
    // - `uci_info`: the `engine crate`
    let uci_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_level(false)
        .with_target(false)
        .with_filter(tracing_subscriber::filter::filter_fn(|meta| {
            meta.target() == "uci" || meta.target() == "uci_info"
        }));

    let stderr_layer = tracing_subscriber::fmt::layer()
        .without_time()
        .with_level(false)
        .with_target(false)
        .with_writer(io::stderr)
        .with_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("info").add_directive(
                    "uci=off"
                        .parse::<tracing_subscriber::filter::Directive>()
                        .unwrap(),
                )
            }),
        );

    let log_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_filter(LevelFilter::from_level(Level::DEBUG));

    Registry::default()
        .with(uci_layer)
        .with(stderr_layer)
        .with(log_layer)
        .init();

    Ok(())
}

fn get_default_log_path() -> Result<PathBuf> {
    let mut logs_dir = dirs::home_dir().context("Home directory not set")?;
    logs_dir.push(PathBuf::from(".local/state/chess"));

    let _ = LOGS_DIRECTORY.get_or_init(|| logs_dir.clone());

    let mut log_path = logs_dir;
    log_path.push("chess.log");
    Ok(log_path)
}
