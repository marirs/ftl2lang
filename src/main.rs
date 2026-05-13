use clap::Parser;
use dialoguer::Confirm;
use ftl2lang::cache::TranslationCache;
use ftl2lang::cli::Args;
use ftl2lang::config::Config;
use ftl2lang::detect::detect_source_language;
use ftl2lang::error::{exit_code, AppError};
use ftl2lang::folder::{collect_ftl_files, target_path_for};
use ftl2lang::ftl::walk::walk_source;
use ftl2lang::lang::{display_name, normalize};
use ftl2lang::pipeline::{translate_file_incremental, Summary};
use ftl2lang::sidecar::{sidecar_path_for, Sidecar};
use ftl2lang::translator::factory::build_translator;
use ftl2lang::translator::Translator;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(exit_code(&e));
    }
}

async fn run() -> Result<(), AppError> {
    let args = Args::parse();
    let target_lang = normalize(&args.to);

    // Load config
    let config_path = args.config.clone().unwrap_or_else(Config::default_path);
    let config = Config::load_from_path(&config_path)?;

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

    if args.input.is_dir() {
        process_folder(&args, &target_lang, translator.as_ref()).await
    } else {
        process_file(&args, &target_lang, translator.as_ref()).await
    }
}

async fn process_file(
    args: &Args,
    target_lang: &str,
    translator: &dyn Translator,
) -> Result<(), AppError> {
    let src = std::fs::read_to_string(&args.input)?;
    let source_lang = resolve_source_lang(args, &src)?;

    // Default output path: sibling file named <target>.ftl
    let out_path = args.out.clone().unwrap_or_else(|| {
        args.input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{}.ftl", target_lang))
    });

    let sidecar_path = sidecar_path_for(&out_path);
    let prev_target = std::fs::read_to_string(&out_path).ok();
    let prev_sidecar = Sidecar::load(&sidecar_path)?;

    if args.dry_run {
        let spans = walk_source(&src, Some(args.input.as_path()))?;
        println!(
            "DRY-RUN: would translate {} text spans in {} via {}",
            spans.len(),
            args.input.display(),
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

    let (out, summary, new_sidecar) = translate_file_incremental(
        &src,
        prev_target.as_deref(),
        &prev_sidecar,
        &source_lang,
        target_lang,
        &cached,
        args.force,
    )
    .await?;

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
    translator: &dyn Translator,
) -> Result<(), AppError> {
    let source_root = &args.input;
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

    // Resolve source lang once from the first file's content (or --from).
    let first_src = std::fs::read_to_string(&files[0])?;
    let source_lang = resolve_source_lang(args, &first_src)?;

    let mut total = Summary::default();
    let mut errors: Vec<(PathBuf, AppError)> = Vec::new();

    for file in &files {
        let out_path = target_path_for(file, source_root, &target_root);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let src = std::fs::read_to_string(file)?;
        let sidecar_path = sidecar_path_for(&out_path);
        let prev_target = std::fs::read_to_string(&out_path).ok();
        let prev_sidecar = Sidecar::load(&sidecar_path)?;

        match translate_file_incremental(
            &src,
            prev_target.as_deref(),
            &prev_sidecar,
            &source_lang,
            target_lang,
            translator,
            args.force,
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

fn resolve_source_lang(args: &Args, src: &str) -> Result<String, AppError> {
    if let Some(from) = &args.from {
        return Ok(normalize(from));
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
    ) -> Result<Vec<String>, AppError> {
        if !self.enabled {
            return self.inner.translate_batch(texts, source_lang, target_lang).await;
        }

        // Determine which texts are cache misses.
        let mut results: Vec<Option<String>> = vec![None; texts.len()];
        let mut to_fetch: Vec<(usize, String)> = Vec::new();
        {
            let cache = self.cache.lock().unwrap();
            for (i, text) in texts.iter().enumerate() {
                if let Some(hit) = cache.get(text, &self.source_lang, &self.target_lang, self.inner.name()) {
                    results[i] = Some(hit);
                } else {
                    to_fetch.push((i, (*text).to_string()));
                }
            }
        }

        if !to_fetch.is_empty() {
            let fetch_texts: Vec<&str> = to_fetch.iter().map(|(_, t)| t.as_str()).collect();
            let translated = self
                .inner
                .translate_batch(&fetch_texts, source_lang, target_lang)
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
