use crate::sidecar::{hash_text, Sidecar};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    New,
    Changed,
    Unchanged,
    Orphaned,
}

/// Classify each message ID in `source_messages` and `target_messages`
/// into one of NEW / CHANGED / UNCHANGED / ORPHANED.
///
/// - `source_messages`: message_id → joined source text (stable serialization).
/// - `target_messages`: message_id → joined target text.
/// - `sidecar`: previous-run hashes per message_id.
///
/// Returns a verdict for every ID present in either map.
pub fn classify(
    source_messages: &BTreeMap<String, String>,
    target_messages: &BTreeMap<String, String>,
    sidecar: &Sidecar,
) -> BTreeMap<String, Verdict> {
    let mut out = BTreeMap::new();

    for (id, src_text) in source_messages {
        let verdict = if target_messages.contains_key(id) {
            let sc_entry = sidecar.entries.get(id);
            match sc_entry {
                // v3 differs: hash mismatch means the source changed since last translation
                Some(entry) if entry.src_hash != hash_text(src_text) => Verdict::Changed,
                // No sidecar entry or hash matches → treat as unchanged to avoid spurious retranslation
                _ => Verdict::Unchanged,
            }
        } else {
            Verdict::New
        };
        out.insert(id.clone(), verdict);
    }

    for id in target_messages.keys() {
        if !source_messages.contains_key(id) {
            out.insert(id.clone(), Verdict::Orphaned);
        }
    }

    out
}
