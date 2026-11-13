//! Matching and expansion: what feelings and needs sit behind a word.

use crate::data::{Catalog, Entry, Kind};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Primary,
    LikelyFeeling,
    LikelyNeed,
    SimilarFeeling,
    FeelingWhenMet,
    FeelingWhenUnmet,
    AlsoMatching,
    DidYouMean,
}

impl Role {
    pub fn heading(self) -> &'static str {
        match self {
            Role::Primary => "Match",
            Role::LikelyFeeling => "Likely feelings",
            Role::LikelyNeed => "Likely needs",
            Role::SimilarFeeling => "Similar feelings",
            Role::FeelingWhenMet => "Feelings when this need is met",
            Role::FeelingWhenUnmet => "Feelings when this need is unmet",
            Role::AlsoMatching => "Also matching",
            Role::DidYouMean => "Did you mean",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Role::Primary => "match",
            Role::LikelyFeeling => "likely-feeling",
            Role::LikelyNeed => "likely-need",
            Role::SimilarFeeling => "similar-feeling",
            Role::FeelingWhenMet => "feeling-when-met",
            Role::FeelingWhenUnmet => "feeling-when-unmet",
            Role::AlsoMatching => "also-matching",
            Role::DidYouMean => "did-you-mean",
        }
    }
}

pub struct Row<'a> {
    pub role: Role,
    pub entry: &'a Entry,
}

const LEAD_INS: [&str; 8] = [
    "i am feeling ",
    "i'm feeling ",
    "i feel like ",
    "i feel ",
    "i am ",
    "i'm ",
    "feeling ",
    "feel ",
];

/// Lowercases, trims and drops a leading "I feel" style phrase.
pub fn clean(query: &str) -> String {
    let lowered = query.trim().to_lowercase();
    for lead in LEAD_INS {
        if let Some(rest) = lowered.strip_prefix(lead) {
            if !rest.trim().is_empty() {
                return rest.trim().to_string();
            }
        }
    }
    lowered
}

/// 0 exact, 1 prefix, 2 word prefix, 3 substring; None when there is no match.
fn rank_name(query: &str, name: &str) -> Option<u8> {
    let n = name.to_lowercase();
    if n == query {
        Some(0)
    } else if n.starts_with(query) {
        Some(1)
    } else if n.split(|c: char| c.is_whitespace() || c == '-').any(|w| w.starts_with(query)) {
        Some(2)
    } else if n.contains(query) {
        Some(3)
    } else {
        None
    }
}

fn kind_order(kind: Kind) -> u8 {
    match kind {
        Kind::Faux => 0,
        Kind::Feeling => 1,
        Kind::Need => 2,
    }
}

/// Entries matching `query` by name (or, with `match_group`, by group name), best first.
pub fn ranked<'a>(entries: &[&'a Entry], query: &str, match_group: bool) -> Vec<&'a Entry> {
    let mut scored: Vec<(u8, u8, usize, &Entry)> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let by_name = rank_name(query, &entry.name);
        let rank = match (by_name, match_group) {
            (Some(r), _) => Some(r),
            (None, true) => rank_name(query, &entry.group).map(|r| r + 10),
            (None, false) => None,
        };
        if let Some(rank) = rank {
            scored.push((rank, kind_order(entry.kind), index, *entry));
        }
    }
    scored.sort_by_key(|s| (s.0, s.1, s.2));
    scored.into_iter().map(|s| s.3).collect()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut current = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            current[j] = (previous[j - 1] + cost)
                .min(previous[j] + 1)
                .min(current[j - 1] + 1);
        }
        previous = current;
    }
    previous[b.len()]
}

/// Closest names to a misspelled `query`, best first.
pub fn suggestions<'a>(entries: &[&'a Entry], query: &str, limit: usize) -> Vec<&'a Entry> {
    let allowed = (query.chars().count() / 4).max(1);
    let mut close: Vec<(usize, &Entry)> = entries
        .iter()
        .filter_map(|e| {
            let d = edit_distance(query, &e.name.to_lowercase());
            (d <= allowed).then_some((d, *e))
        })
        .collect();
    close.sort_by_key(|c| c.0);
    close.into_iter().take(limit).map(|c| c.1).collect()
}

struct Builder<'a> {
    rows: Vec<Row<'a>>,
}

impl<'a> Builder<'a> {
    fn add(&mut self, role: Role, entry: Option<&'a Entry>) {
        if let Some(entry) = entry {
            let duplicate = self
                .rows
                .iter()
                .any(|r| r.entry.name.eq_ignore_ascii_case(&entry.name));
            if !duplicate {
                self.rows.push(Row { role, entry });
            }
        }
    }
}

/// The match itself plus the feelings and needs usually behind it.
pub fn expand<'a>(catalog: &'a Catalog, primary: &'a Entry) -> Vec<Row<'a>> {
    let mut b = Builder { rows: Vec::new() };
    b.add(Role::Primary, Some(primary));
    match primary.kind {
        Kind::Faux => {
            for name in &primary.feelings {
                b.add(Role::LikelyFeeling, catalog.feeling(name));
            }
            for name in &primary.needs {
                b.add(Role::LikelyNeed, catalog.need(name));
            }
        }
        Kind::Feeling => {
            for sibling in catalog.feelings.iter().filter(|f| f.group == primary.group).take(6) {
                b.add(Role::SimilarFeeling, Some(sibling));
            }
            for name in &primary.needs {
                b.add(Role::LikelyNeed, catalog.need(name));
            }
        }
        Kind::Need => {
            let mut shown = 0;
            for met in [false, true] {
                let mut groups: Vec<&str> = Vec::new();
                for f in &catalog.feelings {
                    if f.met == met
                        && f.needs.iter().any(|n| n.eq_ignore_ascii_case(&primary.name))
                        && !groups.contains(&f.group.as_str())
                    {
                        groups.push(f.group.as_str());
                    }
                }
                for group in groups {
                    for f in catalog
                        .feelings
                        .iter()
                        .filter(|f| f.group == group && f.name != f.group)
                        .take(3)
                    {
                        if shown < 8 {
                            let role = if met { Role::FeelingWhenMet } else { Role::FeelingWhenUnmet };
                            let before = b.rows.len();
                            b.add(role, Some(f));
                            shown += b.rows.len() - before;
                        }
                    }
                }
            }
        }
    }
    b.rows
}

/// Full lookup of a word: feelings, needs and faux feelings. Empty when nothing is close.
pub fn lookup<'a>(catalog: &'a Catalog, query: &str) -> Vec<Row<'a>> {
    let query = clean(query);
    if query.is_empty() {
        return Vec::new();
    }
    let everything: Vec<&Entry> = catalog
        .faux
        .iter()
        .chain(catalog.feelings.iter())
        .chain(catalog.needs.iter())
        .collect();
    let found = ranked(&everything, &query, false);
    if found.is_empty() {
        return suggestions(&everything, &query, 5)
            .into_iter()
            .map(|entry| Row { role: Role::DidYouMean, entry })
            .collect();
    }
    let mut rows = expand(catalog, found[0]);
    for entry in found.iter().skip(1).take(11) {
        if !rows.iter().any(|r| r.entry.name.eq_ignore_ascii_case(&entry.name)) {
            rows.push(Row { role: Role::AlsoMatching, entry });
        }
    }
    rows
}

/// Browse feelings or needs, optionally filtered by name or group.
pub fn browse<'a>(entries: &'a [Entry], filter: &str) -> Vec<&'a Entry> {
    let filter = clean(filter);
    let all: Vec<&Entry> = entries.iter().collect();
    if filter.is_empty() {
        return all;
    }
    let found = ranked(&all, &filter, true);
    if found.is_empty() {
        suggestions(&all, &filter, 5)
    } else {
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(rows: &[Row]) -> Vec<String> {
        rows.iter().map(|r| r.entry.name.clone()).collect()
    }

    #[test]
    fn faux_feeling_expands_to_feelings_and_needs() {
        let catalog = Catalog::load();
        let rows = lookup(&catalog, "betrayed");
        let names = titles(&rows);
        assert_eq!(names[0], "Betrayed");
        assert!(names.contains(&"Hurt".to_string()));
        assert!(names.contains(&"Trust".to_string()));
        assert_eq!(rows[0].role, Role::Primary);
    }

    #[test]
    fn offended_is_listed() {
        let catalog = Catalog::load();
        assert_eq!(titles(&lookup(&catalog, "offended"))[0], "Offended");
        assert!(browse(&catalog.needs, "respect").iter().any(|e| e.name == "Respect"));
    }

    #[test]
    fn lead_in_prefix_and_typos() {
        let catalog = Catalog::load();
        assert_eq!(titles(&lookup(&catalog, "I feel offended"))[0], "Offended");
        assert_eq!(titles(&lookup(&catalog, "betrayd"))[0], "Betrayed");
        assert_eq!(lookup(&catalog, "betrayd")[0].role, Role::DidYouMean);
    }

    #[test]
    fn needs_show_related_feelings() {
        let catalog = Catalog::load();
        let rows = lookup(&catalog, "trust");
        assert_eq!(rows[0].entry.kind, Kind::Need);
        assert!(rows.iter().any(|r| r.role == Role::FeelingWhenUnmet));
    }

    #[test]
    fn data_references_resolve() {
        let catalog = Catalog::load();
        for faux in &catalog.faux {
            for f in &faux.feelings {
                assert!(catalog.feeling(f).is_some(), "{} -> feeling {}", faux.name, f);
            }
            for n in &faux.needs {
                assert!(catalog.need(n).is_some(), "{} -> need {}", faux.name, n);
            }
        }
        for feeling in &catalog.feelings {
            for n in &feeling.needs {
                assert!(catalog.need(n).is_some(), "{} -> need {}", feeling.name, n);
            }
        }
    }

    #[test]
    fn browse_filters_by_group() {
        let catalog = Catalog::load();
        let found = browse(&catalog.feelings, "angry");
        assert_eq!(found[0].name, "Angry");
        assert!(found.iter().any(|e| e.name == "Furious"));
    }
}
