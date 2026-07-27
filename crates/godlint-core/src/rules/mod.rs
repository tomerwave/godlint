pub trait Rule {
    type Input;
    type Configuration;
    type Violation;

    const ID: &'static str;

    fn evaluate(
        input: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation>;
}

pub mod function_size;
