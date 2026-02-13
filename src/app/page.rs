use crate::fl;

/// The page to display in the application.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    RightThumb,
    #[default]
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    LeftThumb,
    LeftIndex,
    LeftMiddle,
    LeftRing,
    LeftPinky,
}

impl Page {
    pub fn all() -> &'static [Self] {
        &[
            Self::RightThumb,
            Self::RightIndex,
            Self::RightMiddle,
            Self::RightRing,
            Self::RightPinky,
            Self::LeftThumb,
            Self::LeftIndex,
            Self::LeftMiddle,
            Self::LeftRing,
            Self::LeftPinky,
        ]
    }

    pub fn localized_name(&self) -> String {
        match self {
            Self::RightThumb => fl!("page-right-thumb"),
            Self::RightIndex => fl!("page-right-index-finger"),
            Self::RightMiddle => fl!("page-right-middle-finger"),
            Self::RightRing => fl!("page-right-ring-finger"),
            Self::RightPinky => fl!("page-right-little-finger"),
            Self::LeftThumb => fl!("page-left-thumb"),
            Self::LeftIndex => fl!("page-left-index-finger"),
            Self::LeftMiddle => fl!("page-left-middle-finger"),
            Self::LeftRing => fl!("page-left-ring-finger"),
            Self::LeftPinky => fl!("page-left-little-finger"),
        }
    }

    pub fn as_finger_id(&self) -> &'static str {
        match self {
            Page::RightThumb => "right-thumb",
            Page::RightIndex => "right-index-finger",
            Page::RightMiddle => "right-middle-finger",
            Page::RightRing => "right-ring-finger",
            Page::RightPinky => "right-little-finger",
            Page::LeftThumb => "left-thumb",
            Page::LeftIndex => "left-index-finger",
            Page::LeftMiddle => "left-middle-finger",
            Page::LeftRing => "left-ring-finger",
            Page::LeftPinky => "left-little-finger",
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_all() {
        let pages = Page::all();
        assert_eq!(pages.len(), 10);
        assert_eq!(pages[0], Page::RightThumb);
        assert_eq!(pages[1], Page::RightIndex);
        assert_eq!(pages[2], Page::RightMiddle);
        assert_eq!(pages[3], Page::RightRing);
        assert_eq!(pages[4], Page::RightPinky);
        assert_eq!(pages[5], Page::LeftThumb);
        assert_eq!(pages[6], Page::LeftIndex);
        assert_eq!(pages[7], Page::LeftMiddle);
        assert_eq!(pages[8], Page::LeftRing);
        assert_eq!(pages[9], Page::LeftPinky);
    }

    #[test]
    fn test_page_localized_name() {
        // Check that localized names are not empty.
        // Note: Actual values depend on the loaded translation, which defaults to fallback (English).
        assert!(!Page::RightThumb.localized_name().is_empty());
        assert!(!Page::RightIndex.localized_name().is_empty());
        assert!(!Page::RightMiddle.localized_name().is_empty());
        assert!(!Page::RightRing.localized_name().is_empty());
        assert!(!Page::RightPinky.localized_name().is_empty());
        assert!(!Page::LeftThumb.localized_name().is_empty());
        assert!(!Page::LeftIndex.localized_name().is_empty());
        assert!(!Page::LeftMiddle.localized_name().is_empty());
        assert!(!Page::LeftRing.localized_name().is_empty());
        assert!(!Page::LeftPinky.localized_name().is_empty());
    }
}
