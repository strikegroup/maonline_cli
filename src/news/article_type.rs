#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ArticleType {
    News,
    ShareHoldings,
    TobNews,
}

impl ArticleType {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::News => "news",
            Self::ShareHoldings => "shareholdings",
            Self::TobNews => "tob_news",
        }
    }
}
