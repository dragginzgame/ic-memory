use ic_memory::BootstrapAdmission;

fn prepare(admission: &BootstrapAdmission<'_>) {
    admission.open_memory_by_key("app.journal.v1");
}

fn main() {}
