#![deny(unused_must_use)]

use ic_memory::{AllocationBootstrap, AllocationPolicy, BootstrapError, DeclarationSnapshot};

fn discard<P: AllocationPolicy>(
    bootstrap: &mut AllocationBootstrap<'_>,
    declarations: DeclarationSnapshot,
    policy: &P,
) -> Result<(), BootstrapError<P::Error>> {
    bootstrap.validate_and_commit(declarations, policy)?;
    Ok(())
}

fn main() {}
