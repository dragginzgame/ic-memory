use ic_memory::{MemoryAuthority, SchemaMetadata};

fn main() {
    let _authority = MemoryAuthority {
        authority: "app".to_string(),
        key_prefix: "app.".to_string(),
    };

    let _schema = SchemaMetadata {
        schema_version: None,
    };
}
