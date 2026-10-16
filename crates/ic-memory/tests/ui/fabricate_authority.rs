use ic_memory::{MemoryAuthority};

fn main() {
    let _authority = MemoryAuthority {
        authority: "app".to_string(),
        key_prefix: "app.".to_string(),
    };

}
