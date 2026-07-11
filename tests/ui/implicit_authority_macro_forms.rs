struct Store;

ic_memory::ic_memory_range!(start = 100, end = 109);

fn main() {
    let _ = ic_memory::ic_memory_key!("app.users.v1", Store, 100);
}
