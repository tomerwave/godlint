use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    facts::{CommentFact, CommentKind},
    rules::{CommentRule, Finding, Rule, Violation, evaluate_comment_rule, when_configured},
    source::SourceRange,
    suppression::is_directive_only,
};

pub struct NoCommentedCode;

impl Rule for NoCommentedCode {
    const ID: &'static str = "style/no-commented-code";
    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CommentRule for NoCommentedCode {
    fn check(
        comment: &CommentFact,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        if matches!(comment.kind(), CommentKind::Shebang)
            || is_directive_only(comment.text(), comment.kind())
        {
            return Vec::new();
        }
        let body = comment
            .text()
            .trim_start_matches(['#', '/', '*', ' ', '\t']);
        let code = [
            "if ", "for ", "while ", "return ", "throw ", "raise ", "const ", "let ", "var ",
            "def ", "fn ", "func ", "class ", "import ", "from ", "fmt.",
        ];
        code.iter()
            .any(|prefix| body.starts_with(prefix))
            .then_some((comment.range(), Violation::CommentedCode))
            .into_iter()
            .collect()
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_commented_code.as_ref(), |rule| {
        evaluate_comment_rule::<NoCommentedCode>(facts, rule)
    })
}
