use ic_memory::BootstrapAdmission;

fn prepare(admission: &BootstrapAdmission<'_>) {
    admission.open_memory("app.journal.v1");
}

fn main() {}
