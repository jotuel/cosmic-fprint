// SPDX-License-Identifier: MPL-2.0

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
    DeleteAllUsersPrints,
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
            Self::DeleteAllUsersPrints,
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
            Self::DeleteAllUsersPrints => fl!("page-delete-all-users-prints"),
        }
    }

    pub fn as_finger_id(&self) -> Option<&'static str> {
        match self {
            Page::RightThumb => Some("right-thumb"),
            Page::RightIndex => Some("right-index-finger"),
            Page::RightMiddle => Some("right-middle-finger"),
            Page::RightRing => Some("right-ring-finger"),
            Page::RightPinky => Some("right-little-finger"),
            Page::LeftThumb => Some("left-thumb"),
            Page::LeftIndex => Some("left-index-finger"),
            Page::LeftMiddle => Some("left-middle-finger"),
            Page::LeftRing => Some("left-ring-finger"),
            Page::LeftPinky => Some("left-little-finger"),
            Page::DeleteAllUsersPrints => None,
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
        assert_eq!(pages.len(), 11);
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
        assert_eq!(pages[10], Page::DeleteAllUsersPrints);
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
        assert!(!Page::DeleteAllUsersPrints.localized_name().is_empty());
    }

    #[test]
    fn test_page_as_finger_id() {
        assert_eq!(Page::RightThumb.as_finger_id(), Some("right-thumb"));
        assert_eq!(Page::RightIndex.as_finger_id(), Some("right-index-finger"));
        assert_eq!(Page::RightMiddle.as_finger_id(), Some("right-middle-finger"));
        assert_eq!(Page::RightRing.as_finger_id(), Some("right-ring-finger"));
        assert_eq!(Page::RightPinky.as_finger_id(), Some("right-little-finger"));
        assert_eq!(Page::LeftThumb.as_finger_id(), Some("left-thumb"));
        assert_eq!(Page::LeftIndex.as_finger_id(), Some("left-index-finger"));
        assert_eq!(Page::LeftMiddle.as_finger_id(), Some("left-middle-finger"));
        assert_eq!(Page::LeftRing.as_finger_id(), Some("left-ring-finger"));
        assert_eq!(Page::LeftPinky.as_finger_id(), Some("left-little-finger"));
        assert_eq!(Page::DeleteAllUsersPrints.as_finger_id(), None);
    }
}
