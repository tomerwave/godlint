pub mod analyzers;
pub mod config;
pub mod discovery;
pub mod facts;
pub mod glob;
pub mod paths;
pub mod rules;
pub mod scan;
pub mod source;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
