//! Fast, offline transcript cleanup (filler removal, punctuation, casing).

const FILLERS: &[&str] = &[
    "um", "uh", "uhh", "umm", "er", "ah", "hmm", "hm", "like", "you know", "sort of",
    "kind of", "i mean", "basically", "literally", "actually", "so", "well", "okay",
    "right",
];

const LEADING_PHRASES: &[&str] = &["so ", "well ", "okay ", "yeah ", "right "];

/// Rule-based polish — always available, no extra model download.
pub fn polish_quick(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    let mut s = collapse_whitespace(text.trim());
    s = remove_fillers(&s);
    s = trim_leading_phrases(s);
    s = remove_duplicate_words(&s);
    s = fix_i_pronoun(&s);
    s = capitalize_sentences(&s);
    s = ensure_end_punctuation(&s);
    collapse_whitespace(&s)
}

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_fillers(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut markers = vec![false; lower.len()];
    for filler in FILLERS {
        let mut start = 0;
        while let Some(rel) = lower[start..].find(filler) {
            let i = start + rel;
            let end = i + filler.len();
            let before_ok = i == 0 || !lower.as_bytes()[i - 1].is_ascii_alphanumeric();
            let after_ok = end >= lower.len() || !lower.as_bytes()[end].is_ascii_alphanumeric();
            if before_ok && after_ok {
                for m in &mut markers[i..end] {
                    *m = true;
                }
            }
            start = i + filler.len().max(1);
        }
    }

    text.char_indices()
        .filter(|(i, _)| !markers.get(*i).copied().unwrap_or(false))
        .map(|(_, c)| c)
        .collect()
}

fn trim_leading_phrases(mut text: String) -> String {
    loop {
        let lower = text.to_lowercase();
        let mut trimmed = false;
        for phrase in LEADING_PHRASES {
            if lower.starts_with(phrase) {
                text = text[phrase.len()..].trim_start().to_string();
                trimmed = true;
                break;
            }
        }
        if !trimmed {
            break;
        }
    }
    text
}

fn remove_duplicate_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let mut out = vec![words[0]];
    for w in words.iter().skip(1) {
        if w.to_lowercase() != out.last().unwrap().to_lowercase() {
            out.push(w);
        }
    }
    out.join(" ")
}

fn fix_i_pronoun(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == 'i' && is_word_boundary_before(&chars, i) && is_word_boundary_after(&chars, i) {
            out.push('I');
        } else {
            out.push(chars[i]);
        }
        i += 1;
    }
    out
}

fn is_word_boundary_before(chars: &[char], i: usize) -> bool {
    i == 0 || !chars[i - 1].is_alphanumeric()
}

fn is_word_boundary_after(chars: &[char], i: usize) -> bool {
    i + 1 >= chars.len() || !chars[i + 1].is_alphanumeric()
}

fn capitalize_sentences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut capitalize_next = true;
    for c in text.chars() {
        if capitalize_next && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(c);
            if ".!?".contains(c) {
                capitalize_next = true;
            }
        }
    }
    out
}

fn ensure_end_punctuation(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.ends_with(['.', '!', '?', ',', ':', ';']) {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_fillers() {
        let s = polish_quick("um this is like a test you know");
        assert!(!s.to_lowercase().contains(" um "));
        assert!(!s.to_lowercase().contains(" like "));
    }

    #[test]
    fn fixes_i() {
        assert!(polish_quick("i think so").starts_with("I"));
    }
}
