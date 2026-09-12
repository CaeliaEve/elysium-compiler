use pinyin::ToPinyin;

pub fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn pinyin_syllables(localized_name: &str) -> Vec<String> {
    localized_name
        .chars()
        .filter_map(|character| {
            character
                .to_pinyin()
                .map(|pinyin| pinyin.plain().to_string())
        })
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
}

pub fn build_pinyin_fields(localized_name: &str) -> (String, String) {
    let syllables = pinyin_syllables(localized_name);
    let full = syllables.join("");
    let acronym = syllables
        .iter()
        .filter_map(|part| part.chars().next())
        .collect::<String>();
    (full, acronym)
}
