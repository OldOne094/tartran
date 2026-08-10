/// Shared text helpers used by chunking, word counting, and Arabic post-processing.

pub fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF
    )
}

/// True when the text is predominantly CJK, so we budget/count per-character.
pub fn is_mostly_cjk(text: &str) -> bool {
    let total: usize = text.chars().count();
    if total == 0 {
        return true;
    }
    let cjk: usize = text.chars().filter(|c| is_cjk(*c)).count();
    cjk * 2 >= total
}

/// Best-effort unit count: CJK runs count as one unit per char; Latin counts words.
pub fn measure(s: &str) -> usize {
    let mut cjk = 0usize;
    let mut latin = 0usize;
    let mut prev_latin = false;
    for c in s.chars() {
        if is_cjk(c) {
            cjk += 1;
            prev_latin = false;
        } else if c.is_whitespace() {
            prev_latin = false;
        } else {
            if !prev_latin {
                latin += 1;
            }
            prev_latin = true;
        }
    }
    cjk + latin
}

/// Word/character count persisted on chapters: CJK counts characters, Latin counts words.
pub fn count_units(s: &str) -> i64 {
    measure(s) as i64
}

/// Remove Arabic diacritical marks (tashkeel): harakat, tanween, sukun, shadda, etc.
/// Keeps base letters, digits, and punctuation untouched.
pub fn strip_arabic_diacritics(s: &str) -> String {
    s.chars()
        .filter(|&c| {
            let u = c as u32;
            !(0x064B..=0x065F).contains(&u) && u != 0x0670
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_counts_cjk_chars_and_latin_words() {
        assert_eq!(measure("他睁开眼，看到山峰。"), 10);
        assert_eq!(measure("The sword qi tore through the sky."), 7);
    }

    #[test]
    fn mostly_cjk_detection() {
        assert!(is_mostly_cjk("他睁开了眼睛。"));
        assert!(!is_mostly_cjk("He opened his eyes."));
        assert!(is_mostly_cjk(""));
    }

    #[test]
    fn counts_units_persisted_value() {
        assert_eq!(count_units("他睁开了眼睛。"), 7);
        assert_eq!(count_units("He opened his eyes."), 4);
    }

    #[test]
    fn strips_arabic_diacritics() {
        let s = "كَانَ الْبَطَلُ يَقِفُ هُنَاكَ، مُتَأَهِّبًا";
        let out = strip_arabic_diacritics(s);
        assert_eq!(out, "كان البطل يقف هناك، متأهبا");
        assert!(!out.contains('\u{064E}'));
        assert!(!out.contains('\u{0651}'));
    }

    #[test]
    fn leaves_latin_and_digits_alone() {
        let s = "Chapter 5: The hero stood.";
        assert_eq!(strip_arabic_diacritics(s), s);
    }
}