use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    action: RuleAction,
    #[serde(rename = "os", skip_serializing_if = "Option::is_none")]
    platform: Option<Platform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Features>,
}

impl Rule {
    pub fn test(&self, present_features: Features) -> bool {
        if self
            .features
            .as_ref()
            .is_none_or(|features| present_features.contains(features))
            && self.platform.as_ref().is_none_or(Platform::is_current)
        {
            self.action == RuleAction::Allow
        } else {
            self.action == RuleAction::Disallow
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Features {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_demo_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_custom_resolution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_quick_plays_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_quick_play_singleplayer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_quick_play_multiplayer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_quick_play_realms: Option<bool>,
}

impl Features {
    pub const EMPTY: Self = Self::builder().build();

    pub const fn builder() -> FeaturesBuilder {
        FeaturesBuilder::new()
    }

    pub fn contains(&self, other: &Features) -> bool {
        macro_rules! compare {
            ($field:ident) => {
                match (self.$field, other.$field) {
                    (Some(a), Some(b)) => {
                        if a != b {
                            return false;
                        }
                    }
                    (None, Some(b)) => {
                        if b == true {
                            return false;
                        }
                    }
                    _ => {}
                }
            };
        }

        compare!(is_demo_user);
        compare!(has_custom_resolution);
        compare!(has_quick_plays_support);
        compare!(is_quick_play_singleplayer);
        compare!(is_quick_play_multiplayer);
        compare!(is_quick_play_realms);

        true
    }
}

#[must_use]
pub struct FeaturesBuilder(Features);

impl FeaturesBuilder {
    pub const fn new() -> Self {
        FeaturesBuilder(Features {
            is_demo_user: None,
            has_custom_resolution: None,
            has_quick_plays_support: None,
            is_quick_play_singleplayer: None,
            is_quick_play_multiplayer: None,
            is_quick_play_realms: None,
        })
    }

    pub const fn demo_user(mut self, is_demo_user: bool) -> Self {
        self.0.is_demo_user = Some(is_demo_user);
        self
    }

    pub const fn custom_resolution(mut self, has_custom_resolution: bool) -> Self {
        self.0.has_custom_resolution = Some(has_custom_resolution);
        self
    }

    pub const fn quick_plays_support(mut self, has_quick_plays_support: bool) -> Self {
        self.0.has_quick_plays_support = Some(has_quick_plays_support);
        self
    }

    pub const fn quick_play_singleplayer(mut self, is_quick_play_singleplayer: bool) -> Self {
        self.0.is_quick_play_singleplayer = Some(is_quick_play_singleplayer);
        self
    }

    pub const fn quick_play_multiplayer(mut self, is_quick_play_multiplayer: bool) -> Self {
        self.0.is_quick_play_multiplayer = Some(is_quick_play_multiplayer);
        self
    }

    pub const fn quick_play_realms(mut self, is_quick_play_realms: bool) -> Self {
        self.0.is_quick_play_realms = Some(is_quick_play_realms);
        self
    }

    pub const fn build(self) -> Features {
        self.0
    }
}

impl Default for FeaturesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Platform {
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub os: Option<OperatingSystem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Architecture>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Platform {
    // TODO: Check version. This is good enough for now
    pub fn is_current(&self) -> bool {
        self.os.is_none_or(|os| os == OperatingSystem::CURRENT)
            && self.arch.is_none_or(|arch| arch == Architecture::CURRENT)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingSystem {
    Windows,
    Linux,
    Osx,
}

impl OperatingSystem {
    #[cfg(target_os = "windows")]
    pub const CURRENT: Self = Self::Windows;
    #[cfg(target_os = "linux")]
    pub const CURRENT: Self = Self::Linux;
    #[cfg(target_os = "macos")]
    pub const CURRENT: Self = Self::Osx;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    X64,
    Arm64,
    X86,
    Arm32,
}

impl Architecture {
    #[cfg(target_arch = "x86_64")]
    pub const CURRENT: Self = Self::X64;
    #[cfg(target_arch = "aarch64")]
    pub const CURRENT: Self = Self::Arm64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_platform_current() {
        assert!(
            Platform {
                os: Some(OperatingSystem::CURRENT),
                arch: Some(Architecture::CURRENT),
                version: None,
            }
            .is_current()
        )
    }

    #[test]
    fn features_present() {
        let features = Features::builder()
            .demo_user(true)
            .quick_plays_support(false)
            .build();
        assert!(features.contains(&Features::EMPTY));
        assert!(
            features.contains(&features),
            "Features don't contain themself"
        );
        assert!(
            features.contains(&Features::builder().demo_user(true).build()),
            "Features don't contain demo user"
        );
        assert!(
            features.contains(&Features::builder().custom_resolution(false).build()),
            "Features contain custom resolution"
        );
    }
}
