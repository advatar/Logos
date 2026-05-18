//! Slash bundle verifier CLI.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use slash_verifier::{verify, RegistrySnapshot, SlashBundleFile};

#[derive(Debug, Parser)]
#[command(name = "slash-verifier", about = "LP-0016 slash bundle verifier")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Verify a slash bundle against a registry snapshot.
    Verify {
        /// Path to a JSON file containing a [`RegistrySnapshot`].
        #[arg(long)]
        registry: PathBuf,
        /// Path to a JSON file containing a [`SlashBundleFile`].
        #[arg(long)]
        bundle: PathBuf,
    },
    /// Print the JSON schemas the verifier accepts.
    Schema,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("slash-verifier: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Verify { registry, bundle } => run_verify(&registry, &bundle),
        Command::Schema => {
            println!("RegistrySnapshot {{ forum: ForumConfig, registry: RegistryState }}");
            println!("SlashBundleFile {{ certificates: Vec<ModerationCertificate> }}");
            println!("ForumConfig {{ forum_id, k, n, moderators, mod_set_version, threshold_public_key_hash }}");
            Ok(())
        }
    }
}

fn run_verify(registry_path: &PathBuf, bundle_path: &PathBuf) -> Result<()> {
    let snapshot: RegistrySnapshot = parse_json(registry_path)?;
    let bundle: SlashBundleFile = parse_json(bundle_path)?;
    let result = verify(&snapshot, &bundle)?;
    println!("ok: revoked commitment {}", hex::encode(result.commitment));
    Ok(())
}

fn parse_json<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}
