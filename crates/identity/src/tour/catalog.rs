//! The tours that exist, in the order they auto-open. The server owns ids,
//! versions and step keys; each client owns the anchors and copy for them.
//! Bump a tour's `version` when its steps change so users see it again.

pub struct TourDef {
    pub id: &'static str,
    pub version: u16,
    /// The page/tab the tour runs on (informational for clients).
    pub page: &'static str,
    pub steps: &'static [&'static str],
}

pub const CATALOG: &[TourDef] = &[
    TourDef {
        id: "kitchen",
        version: 1,
        page: "kitchen",
        steps: &["welcome", "nav", "list", "cta"],
    },
    TourDef {
        id: "recipes",
        version: 1,
        page: "recipes",
        steps: &["search", "import", "new", "library"],
    },
    TourDef {
        id: "cooking",
        version: 1,
        page: "kitchen",
        steps: &["up_next", "start", "recipe", "regenerate"],
    },
    TourDef {
        id: "groceries",
        version: 1,
        page: "groceries",
        steps: &["route", "aisle", "check"],
    },
    TourDef {
        id: "settings",
        version: 1,
        page: "settings",
        steps: &["household", "courses", "aisles"],
    },
];

pub fn find(id: &str) -> Option<&'static TourDef> {
    CATALOG.iter().find(|def| def.id == id)
}
