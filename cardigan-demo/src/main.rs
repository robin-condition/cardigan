use cardigan_incremental::{memoized, ReceivedVersioned, Versioned, VersionedComputationInfo};

fn main() {
    println!("Hello, world!");
    let mut t: add = Default::default();

    let mut a = Versioned::default();
    a.set_to_next(Some(&5));
    let mut b = Versioned::default();
    b.set_to_next(Some(2));

    t.compute(&a, &b);
}

#[memoized]
async fn add(a: &i32, b: i32) -> i32 {
    return a + b;
}
