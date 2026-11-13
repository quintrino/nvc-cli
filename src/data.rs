//! The NVC catalog, loaded from the embedded `data/nvc.json`.

use serde::Deserialize;

const DATA: &str = include_str!("../data/nvc.json");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Faux,
    Feeling,
    Need,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Faux => "faux-feeling",
            Kind::Feeling => "feeling",
            Kind::Need => "need",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub kind: Kind,
    pub name: String,
    pub emoji: String,
    pub description: String,
    pub group: String,
    /// Feelings only: whether the group means needs are met.
    pub met: bool,
    /// Faux feelings and feelings: the needs usually behind it.
    pub needs: Vec<String>,
    /// Faux feelings only: the feelings usually behind it.
    pub feelings: Vec<String>,
}

#[derive(Deserialize)]
struct RawEntry {
    name: String,
    emoji: String,
    description: String,
}

#[derive(Deserialize)]
struct RawFeelingGroup {
    name: String,
    met: bool,
    emoji: Option<String>,
    needs: Vec<String>,
    entries: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawNeedGroup {
    name: String,
    entries: Vec<RawEntry>,
}

#[derive(Deserialize)]
struct RawFaux {
    name: String,
    emoji: String,
    description: String,
    feelings: Vec<String>,
    needs: Vec<String>,
}

#[derive(Deserialize)]
struct Raw {
    feeling_groups: Vec<RawFeelingGroup>,
    need_groups: Vec<RawNeedGroup>,
    faux_feelings: Vec<RawFaux>,
}

pub struct Catalog {
    pub feelings: Vec<Entry>,
    pub needs: Vec<Entry>,
    pub faux: Vec<Entry>,
}

impl Catalog {
    pub fn load() -> Catalog {
        let raw: Raw = serde_json::from_str(DATA).expect("embedded data/nvc.json is valid");
        let mut feelings = Vec::new();
        for group in raw.feeling_groups {
            let has_umbrella = group
                .entries
                .iter()
                .any(|e| e.name.eq_ignore_ascii_case(&group.name));
            if !has_umbrella {
                let sample: Vec<String> = group
                    .entries
                    .iter()
                    .take(4)
                    .map(|e| e.name.to_lowercase())
                    .collect();
                feelings.push(Entry {
                    kind: Kind::Feeling,
                    name: group.name.clone(),
                    emoji: group.emoji.clone().unwrap_or_else(|| "•".to_string()),
                    description: format!("Umbrella word covering {} and more.", sample.join(", ")),
                    group: group.name.clone(),
                    met: group.met,
                    needs: group.needs.clone(),
                    feelings: Vec::new(),
                });
            }
            for e in group.entries {
                feelings.push(Entry {
                    kind: Kind::Feeling,
                    name: e.name,
                    emoji: e.emoji,
                    description: e.description,
                    group: group.name.clone(),
                    met: group.met,
                    needs: group.needs.clone(),
                    feelings: Vec::new(),
                });
            }
        }
        let needs = raw
            .need_groups
            .into_iter()
            .flat_map(|group| {
                let name = group.name;
                group.entries.into_iter().map(move |e| Entry {
                    kind: Kind::Need,
                    name: e.name,
                    emoji: e.emoji,
                    description: e.description,
                    group: name.clone(),
                    met: false,
                    needs: Vec::new(),
                    feelings: Vec::new(),
                })
            })
            .collect();
        let faux = raw
            .faux_feelings
            .into_iter()
            .map(|f| Entry {
                kind: Kind::Faux,
                name: f.name,
                emoji: f.emoji,
                description: f.description,
                group: "Not a feeling".to_string(),
                met: false,
                needs: f.needs,
                feelings: f.feelings,
            })
            .collect();
        Catalog { feelings, needs, faux }
    }

    pub fn feeling(&self, name: &str) -> Option<&Entry> {
        self.feelings.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    pub fn need(&self, name: &str) -> Option<&Entry> {
        self.needs.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }
}
