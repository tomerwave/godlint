#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuppressionDefect {
    NoRules,
    UnknownRule { rule: String },
    NotSuppressible { rule: String },
    UnknownOption { option: String },
    RepeatedOption { option: String },
    MissingJustification,
    MissingOwner,
    MissingExpiry,
    InvalidExpiry { value: String },
    Expired { expires: String },
    Unresolved,
}
