//! Explicit installed qualification of the supplied current-graph fixture.
use pocket_ic::{ErrorCode, PocketIcBuilder, RejectResponse};
use std::{env, fs, time::Duration};

fn assert_trap(error: RejectResponse) {
    assert!(
        matches!(
            error.error_code,
            ErrorCode::CanisterTrapped | ErrorCode::CanisterCalledTrap
        ),
        "expected an execution trap, got {error:?}",
    );
}

fn main() {
    let measure_only = match env::args().nth(1).as_deref() {
        None => false,
        Some("--measure-valid-io") => true,
        Some(_) => panic!("expected no arguments or --measure-valid-io"),
    };
    let wasm = fs::read(env::var("IC_MEMORY_QUALIFICATION_WASM").expect("fixture path")).unwrap();
    // The caller owns server startup/teardown. Never implicitly spawn or
    // download a binary as part of qualification.
    let url = env::var("IC_TESTKIT_POCKET_IC_URL").expect("caller-selected PocketIC server URL");
    let pic = PocketIcBuilder::new()
        .with_server_url(url.parse().unwrap())
        .with_application_subnet()
        .build();
    let canister = pic.create_canister();
    pic.add_cycles(canister, 100_000_000_000_000);
    pic.install_canister(canister, wasm.clone(), Vec::new(), None);
    let snapshot = || {
        pic.query_call(canister, canister, "snapshot", Vec::new())
            .unwrap()
    };
    let initial = snapshot();
    assert_eq!(initial.len(), 26);
    assert_eq!(&initial[..8], &1_u64.to_le_bytes());
    assert_eq!(&initial[8..10], &[0x42, 0x6D]);
    assert_eq!(&initial[18..], &1_u64.to_le_bytes());

    if !measure_only {
        // Check all three delegates, including empty spans beyond the extent.
        for operation in 0..3 {
            for (offset, count) in [(65_536, 1), (65_537, 0), (u64::MAX, 1), (u64::MAX - 1, 2)] {
                let mut args = vec![operation, count];
                args.extend_from_slice(&offset.to_le_bytes());
                assert_trap(pic.update_call(canister, canister, "io", args).unwrap_err());
                assert_eq!(
                    snapshot(),
                    initial,
                    "invalid IO must roll back the preceding valid write"
                );
            }
        }
        println!("installed invalid IO: 12 traps, neighboring marker and prior mutation preserved");

        for mode in [1, 2] {
            pic.advance_time(Duration::from_secs(300));
            assert_trap(
                pic.upgrade_canister(canister, wasm.clone(), vec![mode], None)
                    .unwrap_err(),
            );
            assert_eq!(
                snapshot(),
                initial,
                "failed upgrade must preserve original state"
            );
        }
        pic.advance_time(Duration::from_secs(300));
        pic.upgrade_canister(canister, wasm.clone(), vec![0], None)
            .unwrap();
        let upgraded = snapshot();
        assert_eq!(&upgraded[..8], &2_u64.to_le_bytes());
        assert_eq!(&upgraded[8..], &initial[8..]);
        println!(
            "installed upgrade: post-persistence trap and geometry conflict roll back; clean retry advances exactly one generation"
        );
    }

    for operation in 0..3 {
        for (offset, count) in [(7_u64, 1), (65_536, 0), (65_535, 1)] {
            let mut args = vec![operation, count];
            args.extend_from_slice(&offset.to_le_bytes());
            let reply = pic.update_call(canister, canister, "io", args).unwrap();
            assert_eq!(reply.len(), 10);
            let instructions = u64::from_le_bytes(reply[2..].try_into().unwrap());
            println!("valid_io,{operation},{offset},{count},1000,{instructions}");
        }
    }
    let final_snapshot = snapshot();
    assert_eq!(&final_snapshot[8..10], &[0x99, 0x6D]);
    assert_eq!(&final_snapshot[10..], &initial[10..]);
    println!("fixture_wasm_bytes,{}", wasm.len());
}
