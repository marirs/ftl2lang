//! Interactive `--create-config` flow.
//!
//! Reads any existing config, prompts the user per backend, writes the
//! result to disk (`chmod 600` on Unix). API keys are entered through a
//! no-echo password prompt so they don't appear in terminal scrollback.

use crate::config::{Config, DeeplConfig, GoogleConfig, GtranslateConfig};
use crate::error::AppError;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::path::Path;

pub async fn run_interactive_setup(path: &Path) -> Result<(), AppError> {
    let existing = Config::load_from_path(path)?;
    let exists_on_disk = path.exists();

    println!();
    if exists_on_disk {
        println!("Updating existing config at {}", path.display());
        println!();
        print_existing_summary(&existing);
    } else {
        println!("Creating new config at {}", path.display());
    }
    println!();

    let theme = ColorfulTheme::default();

    // Per-backend prompts. Each helper returns the new Option<...Config>.
    let deepl = prompt_deepl(&theme, existing.deepl.as_ref())?;
    let google = prompt_google(&theme, existing.google.as_ref())?;
    let gtranslate = prompt_gtranslate(&theme, existing.gtranslate.is_some())?;

    let default_translator =
        prompt_default_translator(&theme, &existing, &deepl, &google, &gtranslate)?;
    let default_source = prompt_default_source(&theme, existing.default_source.as_deref())?;

    let new_config = Config {
        default_translator,
        default_source,
        deepl,
        google,
        gtranslate,
    };

    write_config_toml(path, &new_config)?;

    println!();
    println!("Wrote {}", path.display());
    #[cfg(unix)]
    println!("(permissions set to 0600 so only you can read your API keys)");
    Ok(())
}

fn print_existing_summary(cfg: &Config) {
    println!("Current values:");
    if let Some(t) = &cfg.default_translator {
        println!("  default_translator = {:?}", t);
    }
    if let Some(s) = &cfg.default_source {
        println!("  default_source     = {:?}", s);
    }
    if let Some(d) = &cfg.deepl {
        println!(
            "  [deepl]      api_key={}  api_url={}",
            mask(d.api_key.as_deref()),
            d.api_url.as_deref().unwrap_or("(default)")
        );
    }
    if let Some(g) = &cfg.google {
        println!(
            "  [google]     api_key={}  project_id={}",
            mask(g.api_key.as_deref()),
            g.project_id.as_deref().unwrap_or("(unset)")
        );
    }
    if cfg.gtranslate.is_some() {
        println!("  [gtranslate] enabled");
    }
}

fn mask(value: Option<&str>) -> String {
    match value {
        Some(v) if !v.is_empty() => "••••••• (set)".to_string(),
        _ => "(unset)".to_string(),
    }
}

fn prompt_deepl(
    theme: &ColorfulTheme,
    existing: Option<&DeeplConfig>,
) -> Result<Option<DeeplConfig>, AppError> {
    let has_key = existing
        .and_then(|d| d.api_key.as_deref())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    let prompt = if has_key {
        "Configure DeepL? (already has a key)"
    } else {
        "Configure DeepL?"
    };
    let configure = Confirm::with_theme(theme)
        .with_prompt(prompt)
        .default(has_key)
        .interact()
        .map_err(prompt_err)?;
    if !configure {
        return Ok(existing.cloned());
    }

    let key_prompt = if has_key {
        "DeepL API key (press Enter to keep existing)"
    } else {
        "DeepL API key"
    };
    let key: String = Password::with_theme(theme)
        .with_prompt(key_prompt)
        .allow_empty_password(has_key)
        .interact()
        .map_err(prompt_err)?;

    let api_key = if key.is_empty() {
        existing.and_then(|d| d.api_key.clone())
    } else {
        Some(key)
    };

    let url_default = existing.and_then(|d| d.api_url.clone()).unwrap_or_default();
    let api_url_input: String = Input::with_theme(theme)
        .with_prompt("DeepL API URL (Enter for free-tier default)")
        .default(url_default)
        .allow_empty(true)
        .interact_text()
        .map_err(prompt_err)?;
    let api_url = if api_url_input.trim().is_empty() {
        None
    } else {
        Some(api_url_input)
    };

    Ok(Some(DeeplConfig { api_key, api_url }))
}

fn prompt_google(
    theme: &ColorfulTheme,
    existing: Option<&GoogleConfig>,
) -> Result<Option<GoogleConfig>, AppError> {
    let has_key = existing
        .and_then(|g| g.api_key.as_deref())
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    let prompt = if has_key {
        "Configure Google Cloud Translation? (already has a key)"
    } else {
        "Configure Google Cloud Translation?"
    };
    let configure = Confirm::with_theme(theme)
        .with_prompt(prompt)
        .default(has_key)
        .interact()
        .map_err(prompt_err)?;
    if !configure {
        return Ok(existing.cloned());
    }

    let key_prompt = if has_key {
        "Google API key (press Enter to keep existing)"
    } else {
        "Google API key"
    };
    let key: String = Password::with_theme(theme)
        .with_prompt(key_prompt)
        .allow_empty_password(has_key)
        .interact()
        .map_err(prompt_err)?;
    let api_key = if key.is_empty() {
        existing.and_then(|g| g.api_key.clone())
    } else {
        Some(key)
    };

    let project_default = existing
        .and_then(|g| g.project_id.clone())
        .unwrap_or_default();
    let project: String = Input::with_theme(theme)
        .with_prompt("Google Cloud project ID")
        .default(project_default)
        .allow_empty(false)
        .interact_text()
        .map_err(prompt_err)?;

    Ok(Some(GoogleConfig {
        api_key,
        project_id: Some(project),
    }))
}

fn prompt_gtranslate(
    theme: &ColorfulTheme,
    existing_enabled: bool,
) -> Result<Option<GtranslateConfig>, AppError> {
    let enable = Confirm::with_theme(theme)
        .with_prompt("Enable gtranslate (free, unofficial)?")
        .default(existing_enabled || true)
        .interact()
        .map_err(prompt_err)?;
    Ok(if enable {
        Some(GtranslateConfig::default())
    } else {
        None
    })
}

fn prompt_default_translator(
    theme: &ColorfulTheme,
    existing: &Config,
    deepl: &Option<DeeplConfig>,
    google: &Option<GoogleConfig>,
    gtranslate: &Option<GtranslateConfig>,
) -> Result<Option<String>, AppError> {
    let mut choices: Vec<&str> = Vec::new();
    if deepl.is_some() {
        choices.push("deepl");
    }
    if google.is_some() {
        choices.push("google");
    }
    if gtranslate.is_some() {
        choices.push("gtranslate");
    }
    choices.push("(no default — must pass --translator each run)");

    if choices.len() == 1 {
        // Only the "no default" entry. Nothing configured.
        return Ok(None);
    }

    // Pre-select the existing default if it's still in the list.
    let default_idx = existing
        .default_translator
        .as_deref()
        .and_then(|cur| choices.iter().position(|c| *c == cur))
        .unwrap_or(0);

    let selection = Select::with_theme(theme)
        .with_prompt("Default translator")
        .items(&choices)
        .default(default_idx)
        .interact()
        .map_err(prompt_err)?;

    let picked = choices[selection];
    if picked.starts_with('(') {
        Ok(None)
    } else {
        Ok(Some(picked.to_string()))
    }
}

fn prompt_default_source(
    theme: &ColorfulTheme,
    existing: Option<&str>,
) -> Result<Option<String>, AppError> {
    let default = existing.unwrap_or("").to_string();
    let s: String = Input::with_theme(theme)
        .with_prompt("Default source language code (e.g. en) — Enter to skip")
        .default(default)
        .allow_empty(true)
        .interact_text()
        .map_err(prompt_err)?;
    Ok(if s.trim().is_empty() { None } else { Some(s) })
}

fn write_config_toml(path: &Path, cfg: &Config) -> Result<(), AppError> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let body = render_config_toml(cfg);
    std::fs::write(path, body)?;
    set_owner_only_permissions(path);
    Ok(())
}

/// Render a Config back to a TOML document the user can re-read or
/// hand-edit. Only sections with content are emitted; unset fields are
/// omitted rather than written as `field = ""`.
pub fn render_config_toml(cfg: &Config) -> String {
    let mut out = String::new();
    out.push_str("# ftl2lang config\n");
    out.push_str("# Generated by `ftl2lang --create-config`. Hand-edits are fine.\n\n");

    if let Some(t) = &cfg.default_translator {
        out.push_str(&format!("default_translator = {}\n", toml_str(t)));
    }
    if let Some(s) = &cfg.default_source {
        out.push_str(&format!("default_source = {}\n", toml_str(s)));
    }
    if cfg.default_translator.is_some() || cfg.default_source.is_some() {
        out.push('\n');
    }

    if let Some(d) = &cfg.deepl {
        out.push_str("[deepl]\n");
        if let Some(k) = &d.api_key {
            out.push_str(&format!("api_key = {}\n", toml_str(k)));
        }
        if let Some(u) = &d.api_url {
            out.push_str(&format!("api_url = {}\n", toml_str(u)));
        }
        out.push('\n');
    }
    if let Some(g) = &cfg.google {
        out.push_str("[google]\n");
        if let Some(k) = &g.api_key {
            out.push_str(&format!("api_key = {}\n", toml_str(k)));
        }
        if let Some(p) = &g.project_id {
            out.push_str(&format!("project_id = {}\n", toml_str(p)));
        }
        out.push('\n');
    }
    if cfg.gtranslate.is_some() {
        out.push_str("[gtranslate]\n");
        out.push('\n');
    }

    out
}

/// Naïve TOML basic-string quoting. Sufficient for API keys / project IDs /
/// language codes; would need extension if we ever wrote arbitrary user
/// text (newlines, etc.).
fn toml_str(s: &str) -> String {
    let escaped: String = s
        .chars()
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '"' => vec!['\\', '"'],
            _ => vec![c],
        })
        .collect();
    format!("\"{}\"", escaped)
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) {
    // No equivalent on non-Unix; the warning issued at load time still applies.
}

fn prompt_err(e: dialoguer::Error) -> AppError {
    // The most common cause is stdin not being a TTY (running through a
    // pipe or in CI). Surface that explicitly so the user can fix it.
    let msg = format!("{}", e);
    if msg.contains("not a terminal") {
        AppError::Other(
            "interactive prompt requires a terminal. \
             Run from a real shell, or write ~/.config/ftl2lang/config.toml by hand."
                .into(),
        )
    } else {
        AppError::Other(format!("prompt: {}", e))
    }
}
