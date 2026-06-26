use pinyin::ToPinyin;

pub fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_search_terms<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let mut terms = values
        .flat_map(|value| {
            normalize_text(value)
                .split(' ')
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms.join(" ")
}

pub fn build_pinyin_fields(localized_name: &str) -> (String, String) {
    let syllables = localized_name
        .chars()
        .filter_map(|character| {
            character
                .to_pinyin()
                .map(|pinyin| pinyin.plain().to_string())
        })
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    let full = syllables.join("");
    let acronym = syllables
        .iter()
        .filter_map(|part| part.chars().next())
        .collect::<String>();
    (full, acronym)
}
