use crate::{
    analyzers::SourceFacts,
    config::{Config, NoCommentsRule, Severity},
    facts::CommentFact,
    rules::{
        CommentRule, Finding, Rule, RuleError, Violation, evaluate_comment_rule, when_configured,
    },
    source::SourceRange,
};

pub struct NoComments;

const SHEBANG: &str = "#!";

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
        if is_shebang(comment) || is_permitted(comment, configuration) {
            return Vec::new();
        }

        vec![(comment.range(), Violation::CommentNotPermitted)]
    }
}

fn is_shebang(comment: &CommentFact) -> bool {
    comment.range().start() == 0 && comment.text().starts_with(SHEBANG)
}

fn is_permitted(comment: &CommentFact, configuration: &NoCommentsRule) -> bool {
    configuration.allow_doc_comments && comment.kind().is_documentation()
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.no_comments.as_ref(), |configuration| {
        evaluate_comment_rule::<NoComments>(facts, configuration)
    })
}
