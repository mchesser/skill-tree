use crate::tree::{Goal, Group, SkillTree, StatusStyle};
use fehler::throws;
use std::io::Write;

impl SkillTree {
    /// Writes graphviz representing this skill-tree to the given output.
    #[throws(anyhow::Error)]
    pub fn write_graphviz(&self, output: &mut dyn Write) {
        write_graphviz(self, output)?
    }

    /// Generates a string containing graphviz content for this skill-tree.
    #[throws(anyhow::Error)]
    pub fn to_graphviz(&self) -> String {
        let mut output = Vec::new();
        write_graphviz(self, &mut output)?;
        String::from_utf8(output)?
    }
}

#[throws(anyhow::Error)]
fn write_graphviz(tree: &SkillTree, output: &mut dyn Write) {
    writeln!(output, r#"digraph g {{"#)?;
    writeln!(output, r#"graph [ rankdir = "LR" ];"#)?;
    writeln!(output, r#"node [ fontsize="16", shape = "ellipse" ];"#)?;
    writeln!(output, r#"edge [ ];"#)?;

    for group in tree.groups() {
        writeln!(output, r#""{}" ["#, group.name)?;
        write_group_label(tree, group, output)?;
        writeln!(output, r#"  shape = "none""#)?;
        writeln!(output, r#"  margin = 0"#)?;
        writeln!(output, r#"]"#)?;
    }

    for goal in tree.goals() {
        writeln!(output, r#""{}" ["#, goal.name)?;
        write_goal_label(goal, output)?;
        writeln!(output, r#"  shape = "note""#)?;
        writeln!(output, r#"  margin = 0"#)?;
        writeln!(output, r#"  style = "filled""#)?;
        writeln!(output, r#"  fillcolor = "darkgoldenrod""#)?;
        writeln!(output, r#"]"#)?;
    }

    for group in tree.groups() {
        if let Some(requires) = &group.requires {
            for requirement in requires {
                writeln!(
                    output,
                    r#"{} -> {};"#,
                    tree.port_name(requirement, "out"),
                    tree.port_name(&group.name, "in"),
                )?;
            }
        }

        for item in group.items() {
            if let Some(requires) = &item.requires {
                for requirement in requires {
                    let port = item
                        .port
                        .as_ref()
                        .ok_or_else(|| anyhow::format_err!("missing port for: {}", item.label))?;

                    writeln!(
                        output,
                        r#"{} -> "{}":_{}_in;"#,
                        tree.port_name(requirement, "out"),
                        group.name,
                        port,
                    )?;
                }
            }
        }
    }

    for goal in tree.goals() {
        if let Some(requires) = &goal.requires {
            for requirement in requires {
                writeln!(
                    output,
                    r#"{} -> {};"#,
                    tree.port_name(requirement, "out"),
                    tree.port_name(&goal.name, "in"),
                )?;
            }
        }
    }

    writeln!(output, r#"}}"#)?;
}


fn escape(s: &str) -> String {
    htmlescape::encode_minimal(s).replace('\n', "<br/>")
}

#[throws(anyhow::Error)]
fn write_goal_label(goal: &Goal, output: &mut dyn Write) {
    let label = goal.label.as_ref().unwrap_or(&goal.name);
    let label = escape(label);
    writeln!(output, r#"  label = "{label}""#, label = label)?;
}

#[throws(anyhow::Error)]
fn write_group_label(tree: &SkillTree, group: &Group, output: &mut dyn Write) {
    writeln!(output, r#"  label = <<table>"#)?;

    let label = group.label.as_ref().unwrap_or(&group.name);
    let label = escape(label);
    let group_href = attribute_str("href", &group.href, "");
    let header_color = group
        .header_color
        .as_ref()
        .map(String::as_str)
        .unwrap_or("darkgoldenrod");

    writeln!(
        output,
        r#"    <tr><td bgcolor="{header_color}" port="all" colspan="2"{group_href}>{label}</td></tr>"#,
        group_href = group_href,
        label = label,
        header_color = header_color
    )?;

    for item in &group.items {
        let item_status = item
            .status
            .as_ref()
            .or(group.status.as_ref())
            .or(tree.default_status.as_ref());

        let mut style = match item_status.and_then(|x| tree.status.get(x)) {
            Some(style) => style.clone(),
            None => StatusStyle::default(),
        };

        let fontcolor = attribute_str("fontcolor", &style.fontcolor, "");
        let bgcolor = attribute_str("bgcolor", &style.bgcolor, "");
        let href = attribute_str("href", &item.href, "");
        if item.href.is_some() && style.start_tag == "" {
            style.start_tag = "<u>".to_owned();
            style.end_tag = "</u>".to_owned();
        }
        let port = item.port.as_ref().map(|port| format!("_{}", port));
        let port_in = attribute_str("port", &port, "_in");
        let port_out = attribute_str("port", &port, "_out");
        writeln!(
            output,
            "    \
             <tr>\
             <td{bgcolor}{port_in}>{emoji}</td>\
             <td{fontcolor}{bgcolor}{href}{port_out}>\
             {start_tag}{label}{end_tag}\
             </td>\
             </tr>",
            fontcolor = fontcolor,
            bgcolor = bgcolor,
            emoji = style.emoji.as_ref().map_or("", String::as_ref),
            href = href,
            port_in = port_in,
            port_out = port_out,
            label = item.label,
            start_tag = style.start_tag,
            end_tag = style.end_tag,
        )?;
    }

    writeln!(output, r#"  </table>>"#)?;
}

fn attribute_str(label: &str, text: &Option<impl AsRef<str>>, suffix: &str) -> String {
    match text {
        None => format!(""),
        Some(t) => format!(" {}=\"{}{}\"", label, t.as_ref(), suffix),
    }
}

impl SkillTree {
    fn port_name(&self, requires: &str, mode: &str) -> String {
        if let Some(index) = requires.find(":") {
            let name = &requires[..index];
            let port = &requires[index + 1..];
            format!(r#""{}":_{}_{}"#, name, port, mode)
        } else if self.is_goal(requires) {
            // Goals don't have ports, so we don't need a `:all`
            format!(r#""{}""#, requires)
        } else {
            format!(r#""{}":all"#, requires)
        }
    }
}
