use fehler::throws;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct SkillTree {
    #[serde(default = "default_status_kinds")]
    pub status: HashMap<String, StatusStyle>,
    #[serde(default = "default_status")]
    pub default_status: Option<String>,
    pub group: Vec<Group>,
    pub goal: Option<Vec<Goal>>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct StatusStyle {
    pub emoji: Option<String>,
    pub bgcolor: Option<String>,
    pub fontcolor: Option<String>,
    #[serde(default)]
    pub start_tag: String,
    #[serde(default)]
    pub end_tag: String,
}

const WATCH_EMOJI: &str = "⌚";
const HAMMER_WRENCH_EMOJI: &str = "🛠️";
const CHECKED_BOX_EMOJI: &str = "☑️";
const RAISED_HAND_EMOJI: &str = "🙋";

fn default_status_kinds() -> HashMap<String, StatusStyle> {
    vec![
        // Can't work on it now
        ("Blocked".to_owned(), StatusStyle {
            emoji: Some(WATCH_EMOJI.to_owned()),
            bgcolor: Some("cornsilk".to_owned()),
            fontcolor: None,
            start_tag: "<i><font color=\"lightgrey\">".to_owned(),
            end_tag: "</font></i>".to_owned(),
        }),

        // Would like to work on it, but need someone
        ("Unassigned".to_owned(), StatusStyle {
            emoji: Some(RAISED_HAND_EMOJI.to_owned()),
            bgcolor: Some("cornsilk".to_owned()),
            fontcolor: Some("red".to_owned()),
            start_tag: "".to_owned(),
            end_tag: "".to_owned(),
        }),

        // People are actively working on it
        ("Assigned".to_owned(), StatusStyle {
            emoji: Some(HAMMER_WRENCH_EMOJI.to_owned()),
            bgcolor: Some("cornsilk".to_owned()),
            fontcolor: None,
            start_tag: "".to_owned(),
            end_tag: "".to_owned(),
        }),

        // This is done!
        ("Complete".to_owned(), StatusStyle {
            emoji: Some(CHECKED_BOX_EMOJI.to_owned()),
            bgcolor: Some("cornsilk".to_owned()),
            fontcolor: None,
            start_tag: "<s>".to_owned(),
            end_tag: "</s>".to_owned(),
        }),
    ]
    .into_iter()
    .collect()
}

fn default_status() -> Option<String> {
    Some("Unassigned".to_owned())
}

#[derive(Debug, Deserialize)]
pub struct Goal {
    pub name: String,
    pub label: Option<String>,
    pub requires: Option<Vec<String>>,
    pub href: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub name: String,
    pub label: Option<String>,
    pub requires: Option<Vec<String>>,
    pub items: Vec<Item>,
    pub width: Option<f64>,
    pub status: Option<String>,
    pub href: Option<String>,
    pub header_color: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GroupIndex(pub usize);

#[derive(Debug, Deserialize)]
pub struct Item {
    pub label: String,
    pub href: Option<String>,
    pub port: Option<String>,
    pub requires: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ItemIndex(pub usize);

impl SkillTree {
    #[throws(anyhow::Error)]
    pub fn load(path: &Path) -> SkillTree {
        let skill_tree_text = std::fs::read_to_string(path)?;
        Self::parse(&skill_tree_text)?
    }

    #[throws(anyhow::Error)]
    pub fn parse(text: &str) -> SkillTree {
        toml::from_str(text)?
    }

    #[throws(anyhow::Error)]
    pub fn validate(&self) {
        // gather: valid requires entries

        for group in &self.group {
            group.validate()?;
        }
    }

    pub fn is_goal(&self, name: &str) -> bool {
        self.goals().any(|goal| goal.name == name)
    }

    pub fn goals(&self) -> impl Iterator<Item = &Goal> {
        self.goal.iter().flat_map(|v| v.iter())
    }

    pub fn groups(&self) -> impl Iterator<Item = &Group> {
        self.group.iter()
    }
}

impl Group {
    #[throws(anyhow::Error)]
    pub fn validate(&self) {
        // check: that `name` is a valid graphviz identifier

        // check: each of the things in requires has the form
        //        `identifier` or `identifier:port` and that all those
        //        identifiers map to groups

        for item in &self.items {
            item.validate()?;
        }
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }
}

impl Item {
    #[throws(anyhow::Error)]
    pub fn validate(&self) {
        // check: each of the things in requires has the form
        //        `identifier` or `identifier:port` and that all those
        //        identifiers map to groups

        // check: if you have a non-empty `requires`, must have a port
    }
}
