use std::path::Path;

use godlint_core::{
    VERSION,
    config::{Config, Severity},
    rules::{Finding, closest_rule_id},
    scan::ScanReport,
    source::{Language, Workflow},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Github,
    Json,
    Sarif,
    Terminal,
}

pub const FORMATS: &str = "github, json, sarif, terminal";

const TOOL_URI: &str = "https://github.com/tomerwave/godlint";

impl Format {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "github" => Some(Self::Github),
            "json" => Some(Self::Json),
            "sarif" => Some(Self::Sarif),
            "terminal" => Some(Self::Terminal),
            _ => None,
        }
    }

    pub const fn is_machine_readable(self) -> bool {
        matches!(self, Self::Json | Self::Sarif)
    }
}

pub fn render(format: Format, findings: &[Finding], report: &ScanReport) -> String {
    match format {
        Format::Github => lines(findings, annotation),
        Format::Json => json(findings, report),
        Format::Sarif => sarif(findings),
        Format::Terminal => lines(findings, terminal),
    }
}

fn lines(findings: &[Finding], render_one: impl Fn(&Finding) -> String) -> String {
    findings
        .iter()
        .map(render_one)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn notices(path: &Path, config: &Config) -> Vec<String> {
    config
        .rules
        .unrecognised()
        .map(|rule| notice(path, rule))
        .collect()
}

fn notice(path: &Path, rule: &str) -> String {
    let suggestion = closest_rule_id(rule)
        .map(|closest| format!(" Did you mean {}?", visible(closest)))
        .unwrap_or_default();

    format!(
        "{}: rules: {} is not a rule godlint {VERSION} knows, so it is ignored.{suggestion}",
        visible(&path.display().to_string()),
        visible(rule)
    )
}

pub fn visible(text: &str) -> String {
    let mut rendered = String::with_capacity(text.len());

    for character in text.chars() {
        push_visible(character, &mut rendered);
    }

    rendered
}

fn push_visible(character: char, rendered: &mut String) {
    match character {
        '\n' => rendered.push_str("\\n"),
        '\r' => rendered.push_str("\\r"),
        '\t' => rendered.push_str("\\t"),
        control if control.is_control() => {
            rendered.push_str(&format!("\\u{{{:04x}}}", control as u32));
        }
        other => rendered.push(other),
    }
}

fn terminal(finding: &Finding) -> String {
    format!(
        "{}:{}:{}: {}[{}] {}",
        visible(&finding.path.display().to_string()),
        finding.line,
        finding.column,
        severity_name(finding.severity),
        finding.rule_id,
        visible(&finding.message())
    )
}

fn annotation(finding: &Finding) -> String {
    format!(
        "::{} file={},line={},col={},title={}::{}",
        annotation_level(finding.severity),
        property(&finding.path.display().to_string()),
        finding.line,
        finding.column,
        property(finding.rule_id),
        message(&finding.message())
    )
}

fn json(findings: &[Finding], report: &ScanReport) -> String {
    let reported = findings.iter().map(json_finding).collect::<Vec<_>>();
    let issues = report
        .issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"path\":{},\"dialect\":{},\"message\":{}}}",
                quoted(&issue.path.display().to_string()),
                quoted(issue_dialect(&issue.path)),
                quoted(&issue.message)
            )
        })
        .collect::<Vec<_>>();

    format!(
        "{{\"version\":{},\"findings\":[{}],\"issues\":[{}]}}",
        quoted(VERSION),
        reported.join(","),
        issues.join(",")
    )
}

fn issue_dialect(path: &Path) -> &'static str {
    if Workflow::names(path) {
        "workflow"
    } else if Language::from_path(path).is_some() {
        "source"
    } else {
        "other"
    }
}

fn json_finding(finding: &Finding) -> String {
    format!(
        concat!(
            "{{\"path\":{},\"line\":{},\"column\":{},",
            "\"severity\":{},\"rule\":{},\"message\":{}}}"
        ),
        quoted(&finding.path.display().to_string()),
        finding.line,
        finding.column,
        quoted(severity_name(finding.severity)),
        quoted(finding.rule_id),
        quoted(&finding.message())
    )
}

fn sarif(findings: &[Finding]) -> String {
    let mut identifiers = findings
        .iter()
        .map(|finding| finding.rule_id)
        .collect::<Vec<_>>();

    identifiers.sort_unstable();
    identifiers.dedup();

    let rules = identifiers
        .iter()
        .map(|identifier| format!("{{\"id\":{}}}", quoted(identifier)))
        .collect::<Vec<_>>();
    let results = findings.iter().map(sarif_result).collect::<Vec<_>>();

    format!(
        concat!(
            "{{\"version\":\"2.1.0\",",
            "\"$schema\":\"https://json.schemastore.org/sarif-2.1.0.json\",",
            "\"runs\":[{{\"tool\":{{\"driver\":{{\"name\":\"godlint\",\"version\":{},",
            "\"informationUri\":{},\"rules\":[{}]}}}},\"results\":[{}]}}]}}"
        ),
        quoted(VERSION),
        quoted(TOOL_URI),
        rules.join(","),
        results.join(",")
    )
}

fn sarif_result(finding: &Finding) -> String {
    format!(
        concat!(
            "{{\"ruleId\":{},\"level\":{},\"message\":{{\"text\":{}}},",
            "\"locations\":[{{\"physicalLocation\":{{\"artifactLocation\":{{\"uri\":{}}},",
            "\"region\":{{\"startLine\":{},\"startColumn\":{}}}}}}}]}}"
        ),
        quoted(finding.rule_id),
        quoted(sarif_level(finding.severity)),
        quoted(&finding.message()),
        quoted(&finding.path.display().to_string().replace('\\', "/")),
        finding.line,
        finding.column
    )
}

fn quoted(text: &str) -> String {
    let mut json = String::with_capacity(text.len() + 2);

    json.push('"');

    for character in text.chars() {
        escape(character, &mut json);
    }

    json.push('"');
    json
}

fn escape(character: char, json: &mut String) {
    match character {
        '"' => json.push_str("\\\""),
        '\\' => json.push_str("\\\\"),
        '\n' => json.push_str("\\n"),
        '\r' => json.push_str("\\r"),
        '\t' => json.push_str("\\t"),
        control if control.is_control() => {
            json.push_str(&format!("\\u{:04x}", control as u32));
        }
        other => json.push(other),
    }
}

fn message(text: &str) -> String {
    visible(text).replace('%', "%25")
}

fn property(text: &str) -> String {
    message(text).replace(':', "%3A").replace(',', "%2C")
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info => "info",
        Severity::Off => "off",
        Severity::Warning => "warning",
    }
}

fn annotation_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info | Severity::Off => "notice",
        Severity::Warning => "warning",
    }
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info | Severity::Off => "note",
        Severity::Warning => "warning",
    }
}
