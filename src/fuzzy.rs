use crate::ucd;
use fuzzy_matcher::clangd::fuzzy_match;
use rayon::prelude::*;

struct ScoredItem<T> {
    item: T,
    score: i64,
}

pub fn prune<'a>(data: &'a [ucd::CharEntry], filter: &str) -> Vec<&'a ucd::CharEntry> {
    if filter.is_empty() {
        return data.par_iter().collect();
    }
    let mut data: Vec<ScoredItem<&ucd::CharEntry>> = data
        .par_iter()
        .filter_map(|x| {
            let str = format!(
                "{} {} {}",
                ucd::CharEntry::fmt_codepoint(x.codepoint),
                x.name,
                x.unicode_1_name
            );
            let score = match fuzzy_match(&str, filter) {
                None => return None,
                Some(score) => score,
            };
            if score <= 0 {
                return None;
            }
            Some(ScoredItem::<&ucd::CharEntry> { item: x, score })
        })
        .collect();
    data.sort_unstable_by_key(|x| x.score);
    data.into_iter().rev().map(|x| x.item).collect::<Vec<_>>()
}
