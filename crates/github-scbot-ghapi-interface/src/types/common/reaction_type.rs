/// GitHub Reaction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhReactionType {
    /// 👍
    PlusOne,
    /// 👎
    MinusOne,
    /// 😄
    Laugh,
    /// 😕
    Confused,
    /// ❤️
    Heart,
    /// 🎉
    Hooray,
    /// 🚀
    Rocket,
    /// 👀
    Eyes,
}

impl GhReactionType {
    /// Convert reaction type to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl From<GhReactionType> for &'static str {
    fn from(reaction_type: GhReactionType) -> &'static str {
        match reaction_type {
            GhReactionType::PlusOne => "+1",
            GhReactionType::MinusOne => "-1",
            GhReactionType::Laugh => "laugh",
            GhReactionType::Confused => "confused",
            GhReactionType::Heart => "heart",
            GhReactionType::Hooray => "hooray",
            GhReactionType::Rocket => "rocket",
            GhReactionType::Eyes => "eyes",
        }
    }
}
