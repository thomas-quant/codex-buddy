use chrono::{Duration, Utc};
use tempfile::tempdir;

use buddy_wrapper::buddy::lifecycle::can_rebirth_at;
use buddy_wrapper::buddy::store::{BuddyStore, PersistedBuddy};
use buddy_wrapper::util::paths::StoragePaths;

#[test]
fn persisted_buddy_round_trips() {
    let dir = tempdir().unwrap();
    let paths = StoragePaths::for_test(dir.path());
    let store = BuddyStore::new(paths).unwrap();
    let buddy = PersistedBuddy::new_for_test("seed-1", "Mochi", "A chaotic little debugger.");
    store.save_global(&buddy).unwrap();
    assert_eq!(store.load_global().unwrap().unwrap(), buddy);
}

#[test]
fn rebirth_requires_fourteen_days() {
    let born = Utc::now();
    assert!(!can_rebirth_at(born, None, born + Duration::days(13)));
    assert!(can_rebirth_at(born, None, born + Duration::days(14)));
}
