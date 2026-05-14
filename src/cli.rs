use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "ftl2lang",
    version,
    about = "Translate Fluent (.ftl) localization files between languages"
)]
pub struct Args {
    /// Path to .ftl file or folder of .ftl files. Required for translation
    /// runs; omit when using --list-langs or --list-translators.
    pub input: Option<PathBuf>,

    /// Target language code (e.g. de, fr, ta). Required for translation
    /// runs; omit when using --list-langs or --list-translators.
    #[arg(long)]
    pub to: Option<String>,

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

    /// List the languages supported by each translator backend and exit.
    /// Does not require --to or <INPUT>.
    #[arg(long)]
    pub list_langs: bool,

    /// List the available translator backends with a one-line description
    /// of each, and exit. Does not require --to or <INPUT>.
    #[arg(long)]
    pub list_translators: bool,

    /// Interactively create or update ~/.config/ftl2lang/config.toml.
    /// Asks per-backend for API keys (no echo), writes a TOML file with
    /// mode 0600. Does not require --to or <INPUT>.
    #[arg(long)]
    pub create_config: bool,

    /// Delete the on-disk translation cache and exit. Reports how many
    /// entries were cleared. Does not require --to or <INPUT>.
    #[arg(long)]
    pub clear_cache: bool,
}

impl Args {
    /// Whether the invocation is a "list-and-exit" query rather than a
    /// translation run. List runs do not need --to or <INPUT>.
    pub fn is_list_query(&self) -> bool {
        self.list_langs || self.list_translators || self.create_config || self.clear_cache
    }

    /// Validate the combination of flags. Translation runs require both
    /// `<INPUT>` and `--to`; list queries do not.
    pub fn validate(&self) -> Result<(), String> {
        if self.is_list_query() {
            return Ok(());
        }
        if self.input.is_none() {
            return Err("missing <INPUT>: required for translation runs".into());
        }
        if self.to.is_none() {
            return Err("missing --to: required for translation runs".into());
        }
        Ok(())
    }
}
