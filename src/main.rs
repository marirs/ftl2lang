use clap::Parser;
use dialoguer::Confirm;
use ftl2lang::cache::TranslationCache;
use ftl2lang::cli::Args;
use ftl2lang::config::Config;
use ftl2lang::detect::detect_source_language;
use ftl2lang::error::{exit_code, AppError};
use ftl2lang::folder::{collect_ftl_files, target_path_for};
use ftl2lang::ftl::walk::walk_source;
use ftl2lang::info::{render_languages, render_translators};
use ftl2lang::lang::{display_name, normalize};
use ftl2lang::pipeline::{translate_file_incremental, Summary};
use ftl2lang::sidecar::{sidecar_path_for, Sidecar};
use ftl2lang::translator::factory::build_translator;
use ftl2lang::translator::Translator;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// indicatif template: shown when we know the total span count.
const PROGRESS_TEMPLATE: &str =
    "  {bar:30.cyan/blue} {pos}/{len} {msg}";

fn make_bar(total: u64, label: &str) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template(PROGRESS_TEMPLATE)
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("=> "),
    );
    bar.set_message(label.to_string());
    // Keep the bar quiet (no inadvertent redraws) when stderr isn't a TTY;
    // indicatif handles that automatically, but throttle redraws too.
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(exit_code(&e));
    }
}

async fn run() -> Result<(), AppError> {
    let args = Args::parse();

    // Query subcommands run before any config / network work. They never
    // need --to or <INPUT>.
    if args.list_translators {
        print!("{}", render_translators());
        return Ok(());
    }
    if args.list_langs {
        print!("{}", render_languages());
        return Ok(());
    }
    if args.create_config {
        let config_path = args.config.clone().unwrap_or_else(Config::default_path);
        ftl2lang::setup::run_interactive_setup(&config_path).await?;
        return Ok(());
    }

    // Translation runs require both --to and <INPUT>.
    args.validate().map_err(AppError::Other)?;
    let target_lang = normalize(args.to.as_deref().expect("validated above"));
    let input = args.input.as_ref().expect("validated above");

    // Load config
    let config_path = args.config.clone().unwrap_or_else(Config::default_path);
    let config = Config::load_from_path(&config_path)?;
    warn_if_config_world_readable(&config_path);

    // Build translator
    let translator = build_translator(args.translator.as_deref(), &config)?;

    // Verify backend supports target.
    if !translator.supports(&target_lang) {
        return Err(AppError::UnsupportedLang {
            backend: translator.name().to_string(),
            lang: target_lang.clone(),
            suggestion: "google".into(),
        });
    }

    if input.is_dir() {
        process_folder(&args, &target_lang, &config, translator.as_ref()).await
    } else {
        process_file(&args, &target_lang, &config, translator.as_ref()).await
    }
}

#[cfg(unix)]
fn warn_if_config_world_readable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mode = meta.permissions().mode();
        // Any bit set for group or other (0o077) means non-owner can read.
        if mode & 0o077 != 0 {
            eprintln!(
                "warning: {} is group/world readable (mode {:o}); chmod 600 to protect API keys",
                path.display(),
                mode & 0o777
            );
        }
    }
}

#[cfg(not(unix))]
fn warn_if_config_world_readable(_path: &Path) {
    // No equivalent permission concept on non-Unix targets.
}

async fn process_file(
    args: &Args,
    target_lang: &str,
    config: &Config,
    translator: &dyn Translator,
) -> Result<(), AppError> {
    let input = args.input.as_ref().expect("validated by run()");
    let src = std::fs::read_to_string(input)?;
    let source_lang = resolve_source_lang(args, &src, config.default_source.as_deref())?;

    // Default output path: sibling file named <target>.ftl
    let out_path = args.out.clone().unwrap_or_else(|| {
        input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}.ftl", target_lang))
    });

    let sidecar_path = sidecar_path_for(&out_path);
    let prev_target = std::fs::read_to_string(&out_path).ok();
    let prev_sidecar = Sidecar::load(&sidecar_path)?;

    if args.dry_run {
        let spans = walk_source(&src, Some(input.as_path()))?;
        println!(
            "DRY-RUN: would translate {} text spans in {} via {}",
            spans.len(),
            input.display(),
            translator.name()
        );
        return Ok(());
    }

    // Set up optional cache.
    let cache_path = TranslationCache::default_path();
    let mut cache = if args.cache {
        TranslationCache::load(&cache_path)?
    } else {
        TranslationCache::default()
    };

    // Caching wrapper around the real translator.
    let cached = CachingTranslator::new(
        translator,
        &mut cache,
        args.cache,
        target_lang.to_string(),
        source_lang.clone(),
    );

    // Build a progress bar with an upper-bound total (every span in the
    // source); the pipeline will narrow `set_length` to the actual count of
    // new/changed spans once it has classified them.
    let initial_total = walk_source(&src, Some(input.as_path()))?.len() as u64;
    let bar = make_bar(initial_total, &format!("via {}", translator.name()));

    let result = translate_file_incremental(
        &src,
        prev_target.as_deref(),
        &prev_sidecar,
        &source_lang,
        target_lang,
        &cached,
        args.force,
        args.prune,
        Some(&bar),
    )
    .await;
    bar.finish_and_clear();
    let (out, summary, new_sidecar) = result?;

    std::fs::write(&out_path, out)?;
    new_sidecar.save(&sidecar_path)?;
    if args.cache {
        cache.save(&cache_path)?;
    }

    print_summary(&out_path, &summary);
    Ok(())
}

async fn process_folder(
    args: &Args,
    target_lang: &str,
    config: &Config,
    translator: &dyn Translator,
) -> Result<(), AppError> {
    let source_root: &Path = args.input.as_deref().expect("validated by run()");
    let target_root = args.out.clone().unwrap_or_else(|| {
        source_root
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(target_lang)
    });

    let files = collect_ftl_files(source_root)?;
    if files.is_empty() {
        println!("No .ftl files found in {}", source_root.display());
        return Ok(());
    }

    if args.dry_run {
        let mut total_spans = 0usize;
        for file in &files {
            let src = std::fs::read_to_string(file)?;
            let spans = walk_source(&src, Some(file.as_path()))?;
            total_spans += spans.len();
            println!("DRY-RUN: {} ({} spans)", file.display(), spans.len());
        }
        println!(
            "DRY-RUN: would translate {} spans across {} files via {}",
            total_spans,
            files.len(),
            translator.name()
        );
        return Ok(());
    }

    // Resolve source lang once from the first file's content (or --from).
    let first_src = std::fs::read_to_string(&files[0])?;
    let source_lang = resolve_source_lang(args, &first_src, config.default_source.as_deref())?;

    // Set up optional cache for folder mode too — biggest win when
    // multiple files share strings.
    let cache_path = TranslationCache::default_path();
    let mut cache = if args.cache {
        TranslationCache::load(&cache_path)?
    } else {
        TranslationCache::default()
    };
    let cached = CachingTranslator::new(
        translator,
        &mut cache,
        args.cache,
        target_lang.to_string(),
        source_lang.clone(),
    );

    let mut total = Summary::default();
    let mut errors: Vec<(PathBuf, AppError)> = Vec::new();

    // MultiProgress: outer bar tracks files processed, inner bar tracks
    // spans within the current file. Both are managed by indicatif so
    // their redraws don't fight each other.
    let multi = MultiProgress::new();
    let outer = multi.add(make_bar(files.len() as u64, "files"));

    for file in &files {
        let out_path = target_path_for(file, source_root, &target_root);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let src = std::fs::read_to_string(file)?;
        let sidecar_path = sidecar_path_for(&out_path);
        let prev_target = std::fs::read_to_string(&out_path).ok();
        let prev_sidecar = Sidecar::load(&sidecar_path)?;

        let initial_total = walk_source(&src, Some(file.as_path()))?.len() as u64;
        let inner = multi.add(make_bar(
            initial_total,
            &format!("{}", file.display()),
        ));

        match translate_file_incremental(
            &src,
            prev_target.as_deref(),
            &prev_sidecar,
            &source_lang,
            target_lang,
            &cached,
            args.force,
            args.prune,
            Some(&inner),
        )
        .await
        {
            Ok((out, summary, new_sc)) => {
                std::fs::write(&out_path, out)?;
                new_sc.save(&sidecar_path)?;
                println!(
                    "{} → {} ({} new, {} unchanged, {} changed)",
                    file.display(),
                    out_path.display(),
                    summary.new,
                    summary.unchanged,
                    summary.changed
                );
                total.new += summary.new;
                total.unchanged += summary.unchanged;
                total.changed += summary.changed;
                total.orphaned += summary.orphaned;
            }
            Err(e) => {
                eprintln!("FAILED {}: {}", file.display(), e);
                errors.push((file.clone(), e));
            }
        }
        inner.finish_and_clear();
        outer.inc(1);
    }
    outer.finish_and_clear();

    // Release the cache borrow held by `cached` before saving.
    drop(cached);
    if args.cache {
        cache.save(&cache_path)?;
    }

    println!(
        "Total: {} new, {} unchanged, {} changed, {} orphaned across {} files; {} failed",
        total.new,
        total.unchanged,
        total.changed,
        total.orphaned,
        files.len(),
        errors.len()
    );

    if !errors.is_empty() {
        return Err(AppError::Other(format!("{} file(s) failed", errors.len())));
    }
    Ok(())
}

fn print_summary(out_path: &Path, summary: &Summary) {
    println!(
        "Wrote {}\n  {} new\n  {} unchanged\n  {} changed\n  {} orphaned",
        out_path.display(),
        summary.new,
        summary.unchanged,
        summary.changed,
        summary.orphaned
    );
}

fn resolve_source_lang(
    args: &Args,
    src: &str,
    config_default: Option<&str>,
) -> Result<String, AppError> {
    // --from has the highest priority. Then config's default_source. Only fall
    // back to whatlang detection (which then asks the user to confirm) if
    // neither is set.
    if let Some(from) = &args.from {
        return Ok(normalize(from));
    }
    if let Some(default) = config_default {
        return Ok(normalize(default));
    }

    // Extract sample text from the source for detection.
    let spans = walk_source(src, None)?;
    let owned: Vec<String> = spans.into_iter().take(20).map(|s| s.text).collect();
    let texts: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
    let detected = detect_source_language(&texts)?;

    if args.yes {
        return Ok(detected.code);
    }

    let name = display_name(&detected.code);
    let confirmed = Confirm::new()
        .with_prompt(format!("Detected source: {} [Y/n]", name))
        .default(true)
        .interact()
        .map_err(|e| AppError::Other(format!("prompt: {}", e)))?;

    if !confirmed {
        return Err(AppError::Other(
            "source language not confirmed; pass --from <LANG>".into(),
        ));
    }
    Ok(detected.code)
}

/// A `Translator` wrapper that consults a `TranslationCache` before delegating,
/// and writes new translations back to the cache. Pass-through when caching
/// is disabled.
struct CachingTranslator<'a> {
    inner: &'a dyn Translator,
    cache: std::sync::Mutex<&'a mut TranslationCache>,
    enabled: bool,
    target_lang: String,
    source_lang: String,
}

impl<'a> CachingTranslator<'a> {
    fn new(
        inner: &'a dyn Translator,
        cache: &'a mut TranslationCache,
        enabled: bool,
        target_lang: String,
        source_lang: String,
    ) -> Self {
        Self {
            inner,
            cache: std::sync::Mutex::new(cache),
            enabled,
            target_lang,
            source_lang,
        }
    }
}

#[async_trait::async_trait]
impl<'a> Translator for CachingTranslator<'a> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn supports(&self, lang: &str) -> bool {
        self.inner.supports(lang)
    }

    async fn translate_batch(
        &self,
        texts: &[&str],
        source_lang: &str,
        target_lang: &str,
        progress: Option<&indicatif::ProgressBar>,
    ) -> Result<Vec<String>, AppError> {
        if !self.enabled {
            return self
                .inner
                .translate_batch(texts, source_lang, target_lang, progress)
                .await;
        }

        // Determine which texts are cache misses.
        let mut results: Vec<Option<String>> = vec![None; texts.len()];
        let mut to_fetch: Vec<(usize, String)> = Vec::new();
        let mut hit_count = 0u64;
        {
            let cache = self.cache.lock().unwrap();
            for (i, text) in texts.iter().enumerate() {
                if let Some(hit) = cache.get(text, &self.source_lang, &self.target_lang, self.inner.name()) {
                    results[i] = Some(hit);
                    hit_count += 1;
                } else {
                    to_fetch.push((i, (*text).to_string()));
                }
            }
        }
        // Cache hits resolve instantly; tick the bar for them up-front so
        // the progress reflects real work done, not just remote work.
        if let Some(bar) = progress {
            if hit_count > 0 {
                bar.inc(hit_count);
            }
        }

        if !to_fetch.is_empty() {
            let fetch_texts: Vec<&str> = to_fetch.iter().map(|(_, t)| t.as_str()).collect();
            let translated = self
                .inner
                .translate_batch(&fetch_texts, source_lang, target_lang, progress)
                .await?;
            let mut cache = self.cache.lock().unwrap();
            for ((idx, text), tr) in to_fetch.iter().zip(translated.iter()) {
                cache.put(text, &self.source_lang, &self.target_lang, self.inner.name(), tr);
                results[*idx] = Some(tr.clone());
            }
        }

        Ok(results.into_iter().map(|o| o.unwrap_or_default()).collect())
    }
}
