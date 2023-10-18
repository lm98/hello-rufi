use rufi_core::core::lang::builtins::{foldhood_plus, mux};
use rufi_core::core::lang::lang::{nbr, rep};
use rufi_core::core::sensor_id::sensor;
use rufi_core::core::vm::round_vm::RoundVM;

pub fn gradient(vm: RoundVM) -> (RoundVM, f64) {
    fn is_source(vm: RoundVM) -> (RoundVM, bool) {
        let val = vm.local_sense::<bool>(&sensor("source")).unwrap().clone();
        (vm, val)
    }

    rep(
        vm,
        |vm1| (vm1, f64::INFINITY),
        |vm2, d| {
            mux(
                vm2,
                is_source,
                |vm4| {
                    (vm4, 0.0 )
                },
                |vm5| {
                    foldhood_plus(
                        vm5,
                        |vm6| (vm6, f64::INFINITY),
                        |a, b| a.min(b),
                        |vm7| {
                            let (vm_, val) = nbr(vm7, |vm8| (vm8, d));
                            (vm_, val + 1.0)
                        }
                    )
                }
            )
        }
    )
}