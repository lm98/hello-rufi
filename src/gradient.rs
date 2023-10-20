use rufi_core::core::lang::builtins::{foldhood_plus, mux};
use rufi_core::core::lang::lang::{nbr, rep};
use rufi_core::core::sensor_id::sensor;
use rufi_core::core::vm::round_vm::RoundVM;
use rufi_core::{lift, foldhood_plus};

pub fn gradient(vm: RoundVM) -> (RoundVM, f64) {
    fn is_source(vm: RoundVM) -> (RoundVM, bool) {
        let val = vm.local_sense::<bool>(&sensor("source")).unwrap().clone();
        (vm, val)
    }

    rep(
        vm,
        lift!(f64::INFINITY),
        |vm1, d| {
            mux(
                vm1,
                is_source,
                lift!(0.0),
                foldhood_plus!(
                    lift!(f64::INFINITY),
                    |a, b| a.min(b),
                    |vm2| {
                        let (vm_, val) = nbr(vm2, lift!(d));
                        (vm_, val + 1.0)
                    }
                )
            )
        }
    )
}