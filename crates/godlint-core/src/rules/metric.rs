use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Metric {
    FunctionLines,
    FileLines,
    BlockDepth,
    ParameterCount,
    Complexity,
    ReturnPaths,
    StatementCount,
    ConditionOperators,
    CognitiveScore,
    ScriptLines,
    JobSteps,
}

impl Metric {
    pub(crate) fn describe(
        self,
        formatter: &mut fmt::Formatter<'_>,
        actual: u32,
        max: u32,
    ) -> fmt::Result {
        match self {
            Self::FunctionLines => {
                write!(
                    formatter,
                    "Function has {actual} effective lines (max {max})."
                )
            }
            Self::FileLines => write!(formatter, "File has {actual} effective lines (max {max})."),
            Self::BlockDepth => write!(
                formatter,
                "Function nests blocks {actual} levels deep (max {max})."
            ),
            Self::ParameterCount => {
                write!(formatter, "Function has {actual} parameters (max {max}).")
            }
            Self::Complexity => write!(
                formatter,
                "Function has decision complexity {actual} (max {max})."
            ),
            Self::ReturnPaths => {
                write!(formatter, "Function has {actual} return paths (max {max}).")
            }
            Self::StatementCount => {
                write!(formatter, "Function has {actual} statements (max {max}).")
            }
            Self::ConditionOperators => {
                write!(
                    formatter,
                    "Condition combines {actual} operators; the limit is {max}."
                )
            }
            Self::CognitiveScore => write!(
                formatter,
                "Function has cognitive complexity {actual} (max {max})."
            ),
            Self::ScriptLines => {
                write!(
                    formatter,
                    "Script has {actual} effective lines (max {max})."
                )
            }
            Self::JobSteps => write!(formatter, "Job has {actual} steps (max {max})."),
        }
    }
}
