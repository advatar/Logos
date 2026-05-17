use clap::Parser;

/// Placeholder CLI. Production should load a forum snapshot plus JSON slash bundle.
#[derive(Debug, Parser)]
struct Args {
    /// Print expected slash-bundle schema.
    #[arg(long)]
    schema: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.schema {
        println!("SlashBundle {{ forum_id, certificates[K] }}");
    } else {
        println!("slash-verifier: wire this to LEZ registry snapshots before submission");
    }
    Ok(())
}
