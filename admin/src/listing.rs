//! How the lists of admin are read: a page at a time, and searched.

/// How many rows a page of a list shows.
pub const PAGE_SIZE: i64 = 50;

/// Where a page stands in its list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pages {
    /// The page shown, from 1.
    pub page: u32,
    /// How many there are: one at least, were it empty.
    pub pages: u32,
    /// How many rows the list holds.
    pub total: u64,
}

impl Pages {
    /// The page `asked` for of a list of `total` rows: the first without
    /// one, the last when the list ends before it.
    pub fn new(asked: Option<u32>, total: u64) -> Self {
        let pages = total.div_ceil(PAGE_SIZE as u64).clamp(1, u32::MAX as u64) as u32;
        Self {
            page: asked.unwrap_or(1).clamp(1, pages),
            pages,
            total,
        }
    }

    /// How many rows come before the page.
    pub fn offset(&self) -> i64 {
        i64::from(self.page - 1) * PAGE_SIZE
    }

    pub fn previous(&self) -> Option<u32> {
        (self.page > 1).then(|| self.page - 1)
    }

    pub fn next(&self) -> Option<u32> {
        (self.page < self.pages).then(|| self.page + 1)
    }
}

/// What is searched for, when something is: `q` without the blanks around
/// it.
pub fn searched(q: Option<&str>) -> Option<&str> {
    q.map(str::trim).filter(|q| !q.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_is_read_a_page_at_a_time() {
        let pages = Pages::new(None, 0);
        assert_eq!((pages.page, pages.pages, pages.offset()), (1, 1, 0));
        assert_eq!((pages.previous(), pages.next()), (None, None));

        let pages = Pages::new(Some(2), 101);
        assert_eq!((pages.page, pages.pages, pages.offset()), (2, 3, 50));
        assert_eq!((pages.previous(), pages.next()), (Some(1), Some(3)));

        // Past the end is the end, and before the start the start.
        assert_eq!(Pages::new(Some(9), 101).page, 3);
        assert_eq!(Pages::new(Some(0), 101).page, 1);
        assert_eq!(Pages::new(Some(3), 100).page, 2);
    }

    #[test]
    fn blanks_search_for_nothing() {
        assert_eq!(searched(Some("  crêpe ")), Some("crêpe"));
        assert_eq!(searched(Some("  ")), None);
        assert_eq!(searched(None), None);
    }
}
