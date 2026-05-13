# ftl2lang

Translate Fluent (`.ftl`) localization files between languages while preserving placeables, selectors, attributes, and developer comments.

## Backends

- **DeepL** — best quality on supported European languages plus JA, KO, ZH. Requires API key.
- **Google Cloud Translation v3** — 130+ languages including TA, HI, TH, AR, HE, FA. Requires API key + project ID.
- **gtranslate** — unofficial free endpoint at `translate.googleapis.com/translate_a/single`. No key. Best-effort; may break without notice.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Single file, autodetect source, gtranslate by default
ftl2lang --to de en.ftl

# Folder, explicit source, no prompt, free backend
ftl2lang --to de --from en --yes en/

# Use DeepL with caching for re-runs
ftl2lang --to de --translator deepl --cache en.ftl

# Tamil via Google Cloud
ftl2lang --to ta --translator google en.ftl

# Preview without calling the API
ftl2lang --to de --from en --dry-run en.ftl
```

## Config

Create `~/.config/ftl2lang/config.toml`:

```toml
default_translator = "deepl"
default_source = "en"

[deepl]
api_key = "your-key"
# api_url = "https://api.deepl.com/v2"   # uncomment for paid tier

[google]
api_key = "your-key"
project_id = "your-project"

[gtranslate]
```

Set `chmod 600 ~/.config/ftl2lang/config.toml` to keep your API keys readable only by you.

## How incremental mode works

When the target `.ftl` already exists, `ftl2lang` preserves your human edits and only translates new or changed messages. State is tracked in a side-car file `<target>.ftl.ftl2lang.json` recording per-message source-text hashes.

Pass `--force` to retranslate everything from scratch. Pass `--prune` to drop messages that no longer exist in the source.

## License

MIT — see [LICENSE](LICENSE).
