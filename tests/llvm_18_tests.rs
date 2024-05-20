#![cfg(feature = "llvm-18-or-greater")]

//! Test that we can parse the copy of `compatibility.ll` in LLVM 18's `test/Bitcode` directory

use llvm_ir::Module;
use std::path::Path;

macro_rules! llvm_test {
    ($path:expr, $func:ident) => {
        #[test]
        #[allow(non_snake_case)]
        fn $func() {
            let _ = env_logger::builder().is_test(true).try_init(); // capture log messages with test harness
            let path = Path::new($path);
            let _ = Module::from_bc_path(&path).expect("Failed to parse module");
        }
    };
}

llvm_test!(
    "tests/llvm_bc/compatibility-as-of-llvm-18.bc",
    compatibility_llvm_18
);

/*
define void @inlineasm(i32 %arg) {
  call i32 asm "bswap $0", "=r,r"(i32 %arg)
  ; CHECK: call i32 asm "bswap $0", "=r,r"(i32 %arg)
  call i32 asm sideeffect "blt $1, $2, $3", "=r,r,rm"(i32 %arg, i32 %arg)
  ; CHECK: call i32 asm sideeffect "blt $1, $2, $3", "=r,r,rm"(i32 %arg, i32 %arg)
  ret void
}
*/

#[test]
fn inline_asm() {
    use std::convert::TryInto;
    use llvm_ir::instruction;

    let _ = env_logger::builder().is_test(true).try_init(); // capture log messages with test harness
    let path = Path::new("tests/llvm_bc/compatibility-as-of-llvm-18.bc");
    let module = Module::from_bc_path(&path).expect("Failed to parse module");
    let func = module
        .get_func_by_name("inlineasm")
        .expect("Failed to find function");
    let bb = func.basic_blocks
        .get(0)
        .expect("expected function to have a basic block");

    let call0 : &instruction::Call = &bb
        .instrs[0]
        .clone()
        .try_into()
        .expect("Expected an Call instruction");

    let call1 : &instruction::Call = &bb
        .instrs[1]
        .clone()
        .try_into()
        .expect("Expected an Call instruction");

    let asm0 : &instruction::InlineAssembly = call0.function
        .as_ref()
        .left()
        .expect("Expected call to be InlineAssembly");

    let asm1 : &instruction::InlineAssembly = call1.function
        .as_ref()
        .left()
        .expect("Expected call to be InlineAssembly");

    assert_eq!(asm0.assembly, "bswap $0");
    assert_eq!(asm0.constraints, "=r,r");
    assert_eq!(asm0.can_unwind, false);
    assert_eq!(asm0.has_side_effects, false);
    assert_eq!(asm0.needs_aligned_stack, false);
    assert_eq!(asm0.dialect, instruction::AssemblyDialect::ATT);

    assert_eq!(asm1.assembly, "blt $1, $2, $3");
    assert_eq!(asm1.constraints, "=r,r,rm");
    assert_eq!(asm1.can_unwind, false);
    assert_eq!(asm1.has_side_effects, true);
    assert_eq!(asm1.needs_aligned_stack, false);
    assert_eq!(asm1.dialect, instruction::AssemblyDialect::ATT);
}

// TODO: Other LLVM-18+ tests
