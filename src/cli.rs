use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "ftl2lang",
    version,
    about = "Translate Fluent (.ftl) localization files between languages"
)]
pub struct Args {
    /// Path to .ftl file or folder of .ftl files.
    pub input: PathBuf,

    /// Target language code (e.g. de, fr, ta).
    #[arg(long)]
    pub to: String,

    /// Source language code. If omitted, auto-detect with Y/n confirmation.
    #[arg(long)]
    pub from: Option<String>,

    /// Translator backend: deepl | google | gtranslate. Default: config, else gtranslate.
    #[arg(long)]
    pub translator: Option<String>,

    /// Output path. File-mode: a .ftl path. Folder-mode: a directory.
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Overwrite target entirely, ignoring existing translations.
    #[arg(long)]
    pub force: bool,

    /// Remove orphaned message IDs from target.
    #[arg(long)]
    pub prune: bool,

    /// Enable on-disk translation cache.
    #[arg(long)]
    pub cache: bool,

    /// Skip the 'Detected: English [Y/n]' confirmation.
    #[arg(long, short = 'y')]
    pub yes: bool,

    /// Show what would be translated; no API call, no write.
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose output.
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Override config file path.
    #[arg(long)]
    pub config: Option<PathBuf>,
}
