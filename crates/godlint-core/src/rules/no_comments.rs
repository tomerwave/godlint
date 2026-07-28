use crate::{
    analyzers::SourceFacts,
    config::{Config, NoCommentsRule, Severity},
    facts::{CommentFact, CommentKind},
    rules::{
        CommentRule, Finding, Rule, RuleError, Violation, evaluate_comment_rule, when_configured,
    },
    source::SourceRange,
};

pub struct NoComments;

impl Rule for NoComments {
    const ID: &'static str = "style/no-comments";

    type Configuration = NoCommentsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CommentRule for NoComments {
    fn check(
        comment: &CommentFact,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        if is_permitted(comment.kind(), configuration) {
            return Vec::new();
        }

        vec![(comment.range(), Violation::CommentNotPermitted)]
    }
}

fn is_permitted(kind: CommentKind, configuration: &NoCommentsRule) -> bool {
    match kind {
        CommentKind::Shebang => true,
        CommentKind::Doc | CommentKind::Docstring => configuration.allow_doc_comments,
        CommentKind::Line | CommentKind::Block => false,
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.no_comments.as_ref(), |configuration| {
        evaluate_comment_rule::<NoComments>(facts, configuration)
    })
}
