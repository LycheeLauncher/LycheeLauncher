use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::rule::{Features, Rule};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arguments {
    Split {
        game: Vec<Argument>,
        jvm: Vec<Argument>,
    },
    Minecraft(String), // only used for 1.12 and below
}

#[derive(Debug)]
pub struct CompiledArguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

impl Arguments {
    pub fn compile(
        self,
        placeholder_formatter: impl Fn(&str) -> Option<String>,
        present_features: Features,
    ) -> CompiledArguments {
        static PLACEHOLDER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\$\{(.*?)\}").expect("Failed to compile placeholder regex")
        });

        fn compile_part(
            arguments: Vec<Argument>,
            placeholder_formatter: impl Fn(&str) -> Option<String>,
            present_features: Features,
        ) -> Vec<String> {
            static VALUE_ARGUMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r"^-[^XD]").expect("Failed to compile value argument regex")
            });

            let mut compiled = Vec::new();

            let mut iter = arguments
                .into_iter()
                .filter_map(|argument| argument.get_if_allowed(present_features))
                .flatten()
                .peekable();
            while let Some(argument) = iter.next() {
                if VALUE_ARGUMENT_REGEX.is_match(&argument) {
                    if let Some(next_argument) =
                        iter.next_if(|next_argument| PLACEHOLDER_REGEX.is_match(next_argument))
                    {
                        if let Some(value) = PLACEHOLDER_REGEX
                            .captures(&next_argument)
                            .and_then(|cap| placeholder_formatter(&cap[1]))
                        {
                            compiled.push(argument);
                            compiled.push(value);
                        }
                    } else {
                        continue;
                    }
                } else {
                    if let Some(found) = PLACEHOLDER_REGEX.find(&argument) {
                        if let Some(cap) = PLACEHOLDER_REGEX.captures(&argument)
                            && let Some(value) = placeholder_formatter(&cap[1])
                        {
                            compiled.push(format!("{}{}", &argument[..found.start()], value));
                        }
                    } else {
                        compiled.push(argument);
                    }
                }
            }

            compiled
        }

        match self {
            Self::Split { game, jvm } => CompiledArguments {
                game: compile_part(game, &placeholder_formatter, present_features),
                jvm: compile_part(jvm, &placeholder_formatter, present_features),
            },
            Self::Minecraft(_) => unimplemented!("Non split arguments not supported"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Ruled {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

impl Argument {
    pub fn get_if_allowed(self, present_features: Features) -> Option<Vec<String>> {
        match self {
            Self::String(value) => Some(vec![value]),
            Self::Ruled { rules, value } => {
                if rules.iter().all(|rule| rule.test(present_features)) {
                    Some(value.raw())
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

impl ArgumentValue {
    pub fn raw(self) -> Vec<String> {
        match self {
            Self::Single(value) => vec![value],
            Self::Multiple(values) => values,
        }
    }
}
