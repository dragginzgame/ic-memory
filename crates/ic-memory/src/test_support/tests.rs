use super::*;

#[test]
fn clones_share_observations_and_reset_preserves_storage() {
    let memory = Metered::default();
    let observer = memory.clone();
    assert_eq!(memory.grow(1), 0);
    memory.write(0, b"ok");
    let mut read = [0; 2];
    observer.read(0, &mut read);
    assert_eq!(&read, b"ok");
    assert_eq!(memory.counts(), observer.counts());
    assert_eq!(
        observer.counts(),
        Counts {
            reads: 1,
            read_bytes: 2,
            writes: 1,
            write_bytes: 2,
            grows: 1,
            grown_pages: 1,
            overflowed: false
        }
    );
    observer.reset();
    assert_eq!(memory.counts(), Counts::default());
    assert_eq!(memory.size(), 1);
    observer.read(0, &mut read);
    assert_eq!(&read, b"ok");
    assert_eq!(memory.counts().writes, 0);
    memory.write(2, &[]);
    assert_eq!(observer.counts().writes, 1);
    assert_eq!(observer.counts().write_bytes, 0);
}

#[test]
fn actual_overflow_is_sticky_and_reset_opens_an_exact_window() {
    let memory = Metered::default();
    memory.grow(1);
    memory.counts.set(Counts {
        writes: u64::MAX - 1,
        write_bytes: u64::MAX - 1,
        ..Counts::default()
    });
    memory.write(0, b"x");
    assert_eq!(memory.counts().writes, u64::MAX);
    assert!(!memory.counts().overflowed);
    memory.write(0, b"x");
    assert_eq!(memory.counts().writes, u64::MAX);
    assert_eq!(memory.counts().write_bytes, u64::MAX);
    assert!(memory.counts().overflowed);
    memory.read(0, &mut [0]);
    assert!(memory.counts().overflowed);
    memory.reset();
    assert_eq!(memory.counts(), Counts::default());
}
