use std::cmp::Ordering;
use std::sync::OnceLock;

use icu_collator::options::CollatorOptions;
use icu_collator::{Collator, CollatorBorrowed};

fn collator() -> &'static CollatorBorrowed<'static> {
    static COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();
    COLLATOR.get_or_init(|| Collator::try_new(Default::default(), CollatorOptions::default()).expect("root collation data is compiled in"))
}

/// `a.localeCompare(b)` in the `en-US` locale Node and Workers default to, whose collation is ICU's root order.
pub fn locale_compare(a: &str, b: &str) -> Ordering {
    collator().compare(a, b)
}
