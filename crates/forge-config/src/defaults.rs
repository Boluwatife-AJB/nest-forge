/// Default values for the forge configuration.
pub struct ConfigDefaults;

impl ConfigDefaults {
    pub fn source_root() -> &'static str { "src" }
    pub fn language() -> &'static str { "ts" }
    pub fn generate_spec() -> bool { true }
    pub fn flat() -> bool { false }
}