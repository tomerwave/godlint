const ATTACKER_CONTEXTS: [&str; 20] = [
    "github.event.issue.title",
    "github.event.issue.body",
    "github.event.pull_request.title",
    "github.event.pull_request.body",
    "github.event.pull_request.head.ref",
    "github.event.comment.body",
    "github.event.review.body",
    "github.event.review_comment.body",
    "github.event.discussion.title",
    "github.event.discussion.body",
    "github.event.head_commit.message",
    "github.event.head_commit.author.name",
    "github.event.head_commit.author.email",
    "github.event.commits",
    "github.event.pages",
    "github.event.pull_request.head.label",
    "github.event.pull_request.head.repo.default_branch",
    "github.event.workflow_run.head_branch",
    "github.event.workflow_run.head_commit.message",
    "github.head_ref",
];

pub(crate) fn is_attacker_influenced(context: &str) -> bool {
    matches_context(&ATTACKER_CONTEXTS, context)
}

pub(crate) fn matches_context(contexts: &[&str], context: &str) -> bool {
    contexts.iter().any(|candidate| {
        context == *candidate || candidate.ends_with('.') && context.starts_with(candidate)
    })
}
