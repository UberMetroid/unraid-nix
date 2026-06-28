//! Hand-written YAML serializer/parser for process-compose configuration files.
//!
//! Replaces `serde_yml` to eliminate the `libyml` C dependency (which carries
//! known CVEs). The schema we round-trip is small and stable:
//!   - `version: "0.5"`
//!   - `environment: [string, ...]`
//!   - `log_configuration: { add_timestamp, fields_order, rotation }`
//!   - `processes: { name: { command, availability, environment, log_location, log_configuration } }`
//!
//! The output format mirrors what `serde_yml::to_string` previously emitted so
//! files already on disk continue to parse. Process-compose (the external CLI)
//! accepts this standard YAML.

mod convert;
mod parse;
mod scalar;
mod serialize;
mod value;

#[cfg(test)]
mod common;
#[cfg(test)]
mod tests_parse;
#[cfg(test)]
mod tests_serialize;

pub use parse::parse_config;
pub use serialize::serialize_config;
