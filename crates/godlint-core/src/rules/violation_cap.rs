use super::{Severity, Violation};

impl Violation {
    #[rustfmt::skip]
    pub(crate) fn cap(&self) -> Severity {
        match self {
            Self::InternalImport { certain: false, .. } => Severity::Warning,
            Self::UnlabelledActionPin { .. } | Self::StepContinuesOnError | Self::JobContinuesOnError => Severity::Warning,
            Self::ScriptOrTrue | Self::MissingAssertion | Self::UnverifiedHash { .. } => Severity::Warning,
            Self::TemplateInjection { certain: false, .. } => Severity::Warning,
            Self::Limit { .. } | Self::EmptyBody | Self::EmptyErrorHandler => Severity::Error,
            Self::MissingReference { .. } | Self::CommentNotPermitted | Self::WorkflowCommentNotPermitted => Severity::Error,
            Self::UnaccountableSuppression { .. } | Self::UnusedSuppression | Self::RestrictedCall { .. } => Severity::Error,
            Self::DynamicExecution { .. } | Self::DirectEnvironmentRead { .. } | Self::TimerWithoutDelay { .. } => Severity::Error,
            Self::ProductionLog { .. } | Self::InsecureRandom { .. } | Self::WeakHash { .. } => Severity::Error,
            Self::FocusedTest | Self::SkippedTest | Self::EmptyTest => Severity::Error,
            Self::ShellCommand { .. } | Self::UndeclaredPermissions | Self::InheritedPermissions { .. } => Severity::Error,
            Self::HardcodedContainerCredential { .. } | Self::MutableActionReference { .. } | Self::ContradictoryActionLabels { .. } => Severity::Error,
            Self::ContradictoryActionPins { .. } | Self::TemplateInjection { certain: true, .. } | Self::AttackerInfluencedBotCondition { .. } => Severity::Error,
            Self::InheritedSecrets { .. } | Self::OverprovisionedSecrets { .. } | Self::UnredactedSecret => Severity::Error,
            Self::UntrustedGithubEnv { .. } => Severity::Error,
            Self::ScriptExitsSuccessfully { .. } | Self::TestHelperInProduction { .. } | Self::InternalImport { certain: true, .. } => Severity::Error,
            Self::SleepInTest { .. } | Self::UnseededRandom { .. } | Self::NetworkInUnitTest { .. } | Self::NetworkTimeoutMissing { .. } | Self::NoControlFlowInFinally | Self::RedundantCatchRethrow | Self::CommittedSecretFile | Self::CommentedCode | Self::DuplicateString { .. } => Severity::Error,
            Self::RestrictedImport { .. } | Self::CrossedBoundary { .. } | Self::BrokeIndependence { .. } => Severity::Error,
            Self::ForbiddenDependency { .. } | Self::FilenameCase { .. } | Self::InvalidBranchName { .. } => Severity::Error,
            Self::DependencyPolicy { .. } => Severity::Error,
        }
    }
}
