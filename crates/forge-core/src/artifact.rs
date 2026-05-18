use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use serde::Serialize;
use std::path::PathBuf;

use crate::error::{ForgeError, ForgeResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactKind {
    Module,
    Service,
    Controller,
    Class,
    Dto,
    Guard,
    Interceptor,
    Middleware,
    Pipe,
    Decorator,
    Strategy,
    Interface,
    Filter,
    Config,
    Resolver,
    Entity,
}

impl ArtifactKind {
    /// Parse from a CLI string, including aliases.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "module" | "mo" => Some(Self::Module),
            "service" | "s" => Some(Self::Service),
            "controller" | "co" => Some(Self::Controller),
            "class" | "cl" => Some(Self::Class),
            "dto" => Some(Self::Dto),
            "guard" | "gu" => Some(Self::Guard),
            "interceptor" | "itc" => Some(Self::Interceptor),
            "interface" | "itf" => Some(Self::Interface),
            "middleware" | "mi" => Some(Self::Middleware),
            "pipe" | "p" => Some(Self::Pipe),
            "decorator" | "d" => Some(Self::Decorator),
            "strategy" => Some(Self::Strategy),
            "filter" | "f" => Some(Self::Filter),
            "config" => Some(Self::Config),
            "resolver" | "r" => Some(Self::Resolver),
            "entity" | "e" => Some(Self::Entity),
            _ => None,
        }
    }

    /// The template directory name for this artifact.
    pub fn template_name(&self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Service => "service",
            Self::Controller => "controller",
            Self::Class => "class",
            Self::Dto => "dto",
            Self::Guard => "guard",
            Self::Interceptor => "interceptor",
            Self::Middleware => "middleware",
            Self::Pipe => "pipe",
            Self::Decorator => "decorator",
            Self::Strategy => "strategy",
            Self::Interface => "interface",
            Self::Filter => "filter",
            Self::Config => "config",
            Self::Resolver => "resolver",
            Self::Entity => "entity",
        }
    }

    /// All valid names and aliases used for "did you mean?" suggestions.
    pub fn all_names() -> &'static [&'static str] {
        &[
            "module",
            "mo",
            "service",
            "s",
            "controller",
            "co",
            "class",
            "cl",
            "dto",
            "guard",
            "gu",
            "interceptor",
            "itc",
            "interface",
            "itf",
            "middleware",
            "mi",
            "pipe",
            "pi",
            "decorator",
            "d",
            "strategy",
            "filter",
            "f",
            "config",
            "resolver",
            "r",
            "entity",
            "e",
        ]
    }
}

/// All the name variants derived from the user's input.
/// This is computed once and passed to the template engine.
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactName {
    pub raw: String,
    pub pascal: String,
    pub camel: String,
    pub snake: String,
    pub kebab: String,
}

impl ArtifactName {
    /// Parse and validate a raw name string
    /// Return ForgeError::InvalidName with a specific reason on failure.
    pub fn parse(raw: &str) -> ForgeResult<Self> {
        validate_name(raw)?;
        Ok(Self::new_unchecked(raw))
    }

    /// Internal constructor skips validation
    pub(crate) fn new_unchecked(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
            pascal: raw.to_pascal_case(),
            camel: raw.to_lower_camel_case(),
            snake: raw.to_snake_case(),
            kebab: raw.to_kebab_case(),
        }
    }
}

fn validate_name(name: &str) -> ForgeResult<()> {
    if name.is_empty() {
        return Err(ForgeError::InvalidName {
            name: name.to_string(),
            reason: "name cannot be empty".into(),
        });
    }

    if name.contains(' ') {
        return Err(ForgeError::InvalidName {
            name: name.to_string(),
            reason: "name cannot contain spaces: use hyphens instead.".into(),
        });
    }

    // Must start with a unicode letter
    if !name
        .chars()
        .next()
        .map(|c| c.is_alphabetic())
        .unwrap_or(false)
    {
        return Err(ForgeError::InvalidName {
            name: name.to_string(),
            reason: "name must start with a letter".into(),
        });
    }

    // Must start with a unicode letter
    if !name
        .chars()
        .next()
        .map(|c| c.is_alphanumeric())
        .unwrap_or(false)
    {
        return Err(ForgeError::InvalidName {
            name: name.to_string(),
            reason: "name must start with a letter".into(),
        });
    }

    // Only allow alphanumeric, hyphens and underscores
    let invalid_char = name
        .chars()
        .find(|c| !c.is_alphanumeric() && *c != '-' && *c != '_');

    if let Some(c) = invalid_char {
        return Err(ForgeError::InvalidName {
            name: name.to_string(),
            reason: format!(
                "name contains invalid '{c}' - only letters, digits, underscores, and hyphens are allowed."
            ),
        });
    }

    if name.len() > 128 {
        return Err(ForgeError::InvalidName {
            name: name.to_string(),
            reason: "name is too long (maximum 128 characters)".into(),
        });
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub kind: ArtifactKind,
    pub name: ArtifactName,
    pub output_path: PathBuf, // resolved absolute path
    pub flat: bool,
    pub dry_run: bool,
    pub generate_spec: bool,
}
