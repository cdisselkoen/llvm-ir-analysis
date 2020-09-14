use itertools::Itertools;
use llvm_ir::{Module, Name};
use llvm_ir_analysis::*;

fn init_logging() {
    // capture log messages with test harness
    let _ = env_logger::builder().is_test(true).try_init();
}

const HAYBALE_BASIC_BC_PATH: &'static str = "../haybale/tests/bcfiles/basic.bc";

/// Function names in haybale's basic.bc
const FUNC_NAMES: &'static [&'static str] = &[
    "no_args_zero",
    "no_args_nozero",
    "one_arg",
    "two_args",
    "three_args",
    "four_args",
    "five_args",
    "binops",
    "conditional_true",
    "conditional_false",
    "conditional_nozero",
    "conditional_with_and",
    "has_switch",
    "int8t",
    "int16t",
    "int32t",
    "int64t",
    "mixed_bitwidths",
];

#[test]
fn call_graph() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let callgraph = analysis.call_graph();

    // none of these functions have calls or are called
    for func_name in FUNC_NAMES {
        assert_eq!(callgraph.callers(func_name).count(), 0);
        assert_eq!(callgraph.callees(func_name).count(), 0);
    }
}

#[test]
fn functions_by_type() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let fbt = analysis.functions_by_type();

    let functy = module.types.func_type(
        module.types.void(),
        vec![],
        false,
    );
    assert_eq!(fbt.functions_with_type(&functy).count(), 0);

    let functy = module.types.func_type(
        module.types.i32(),
        vec![],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["no_args_nozero", "no_args_zero"]);

    let functy = module.types.func_type(
        module.types.i32(),
        vec![module.types.i32()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["one_arg"]);

    let functy = module.types.func_type(
        module.types.i32(),
        vec![module.types.i32(), module.types.i32()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec![
        "binops",
        "conditional_false",
        "conditional_nozero",
        "conditional_true",
        "conditional_with_and",
        "has_switch",
        "int32t",
        "two_args",
    ]);

    let functy = module.types.func_type(
        module.types.i32(),
        vec![module.types.i32(), module.types.i32(), module.types.i32()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["three_args"]);

    let functy = module.types.func_type(
        module.types.i32(),
        vec![module.types.i32(), module.types.i32(), module.types.i32(), module.types.i32()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["four_args"]);

    let functy = module.types.func_type(
        module.types.i32(),
        vec![module.types.i32(), module.types.i32(), module.types.i32(), module.types.i32(), module.types.i32()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["five_args"]);

    let functy = module.types.func_type(
        module.types.i8(),
        vec![module.types.i8(), module.types.i8()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["int8t"]);

    let functy = module.types.func_type(
        module.types.i16(),
        vec![module.types.i16(), module.types.i16()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["int16t"]);

    let functy = module.types.func_type(
        module.types.i64(),
        vec![module.types.i64(), module.types.i64()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["int64t"]);

    let functy = module.types.func_type(
        module.types.i64(),
        vec![module.types.i8(), module.types.i16(), module.types.i32(), module.types.i64()],
        false,
    );
    let func_names: Vec<&str> = fbt.functions_with_type(&functy).sorted().collect();
    assert_eq!(func_names, vec!["mixed_bitwidths"]);
}

/// Get the name of the entry block in the given function
fn get_entry_block_name<'m>(module: &'m Module, funcname: &str) -> &'m Name {
    let func = module.get_func_by_name(funcname).unwrap();
    &func.basic_blocks[0].name
}

#[test]
fn trivial_cfgs() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);

    for func_name in &[
        "no_args_zero",
        "no_args_nozero",
        "one_arg",
        "two_args",
        "three_args",
        "four_args",
        "five_args",
        "binops",
        "conditional_with_and",
        "int8t",
        "int16t",
        "int32t",
        "int64t",
        "mixed_bitwidths",
    ] {
        let cfg = analysis.control_flow_graph(func_name);
        let entry = get_entry_block_name(&module, func_name);
        assert_eq!(cfg.preds(entry).count(), 0);
        assert_eq!(cfg.succs(entry).count(), 0);
    }
}

#[test]
fn conditional_true_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("conditional_true");

    let bb2_name = Name::from(2);
    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<&Name> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![&Name::from(4), &Name::from(8)]);

    let bb4_name = Name::from(4);
    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&Name::from(2)]);
    let bb4_succs: Vec<&Name> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![&Name::from(12)]);

    let bb8_name = Name::from(8);
    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&Name::from(2)]);
    let bb8_succs: Vec<&Name> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![&Name::from(12)]);

    let bb12_name = Name::from(12);
    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&Name::from(4), &Name::from(8)]);
    let bb12_succs: Vec<&Name> = cfg.succs(&bb12_name).sorted().collect();
    assert!(bb12_succs.is_empty());
}

#[test]
fn conditional_false_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("conditional_false");

    let bb2_name = Name::from(2);
    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<&Name> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![&Name::from(4), &Name::from(8)]);

    let bb4_name = Name::from(4);
    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&Name::from(2)]);
    let bb4_succs: Vec<&Name> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![&Name::from(12)]);

    let bb8_name = Name::from(8);
    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&Name::from(2)]);
    let bb8_succs: Vec<&Name> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![&Name::from(12)]);

    let bb12_name = Name::from(12);
    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&Name::from(4), &Name::from(8)]);
    let bb12_succs: Vec<&Name> = cfg.succs(&bb12_name).sorted().collect();
    assert!(bb12_succs.is_empty());
}

#[test]
fn condtional_nozero_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("conditional_nozero");

    let bb2_name = Name::from(2);
    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<&Name> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![&Name::from(4), &Name::from(14)]);

    let bb4_name = Name::from(4);
    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&Name::from(2)]);
    let bb4_succs: Vec<&Name> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![&Name::from(6), &Name::from(8)]);

    let bb6_name = Name::from(6);
    let bb6_preds: Vec<&Name> = cfg.preds(&bb6_name).sorted().collect();
    assert_eq!(bb6_preds, vec![&Name::from(4)]);
    let bb6_succs: Vec<&Name> = cfg.succs(&bb6_name).sorted().collect();
    assert_eq!(bb6_succs, vec![&Name::from(14)]);

    let bb8_name = Name::from(8);
    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&Name::from(4)]);
    let bb8_succs: Vec<&Name> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![&Name::from(10), &Name::from(12)]);

    let bb10_name = Name::from(10);
    let bb10_preds: Vec<&Name> = cfg.preds(&bb10_name).sorted().collect();
    assert_eq!(bb10_preds, vec![&Name::from(8)]);
    let bb10_succs: Vec<&Name> = cfg.succs(&bb10_name).sorted().collect();
    assert_eq!(bb10_succs, vec![&Name::from(14)]);

    let bb12_name = Name::from(12);
    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&Name::from(8)]);
    let bb12_succs: Vec<&Name> = cfg.succs(&bb12_name).sorted().collect();
    assert_eq!(bb12_succs, vec![&Name::from(14)]);

    let bb14_name = Name::from(14);
    let bb14_preds: Vec<&Name> = cfg.preds(&bb14_name).sorted().collect();
    assert_eq!(bb14_preds, vec![&Name::from(2), &Name::from(6), &Name::from(10), &Name::from(12)]);
    let bb14_succs: Vec<&Name> = cfg.succs(&bb14_name).sorted().collect();
    assert!(bb14_succs.is_empty());
}

#[test]
fn has_switch_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("has_switch");

    let bb2_name = Name::from(2);
    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<&Name> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![
        &Name::from(4),
        &Name::from(5),
        &Name::from(7),
        &Name::from(10),
        &Name::from(11),
        &Name::from(12),
        &Name::from(14),
    ]);

    let bb4_name = Name::from(4);
    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&Name::from(2)]);
    let bb4_succs: Vec<&Name> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![&Name::from(14)]);

    let bb5_name = Name::from(5);
    let bb5_preds: Vec<&Name> = cfg.preds(&bb5_name).sorted().collect();
    assert_eq!(bb5_preds, vec![&Name::from(2)]);
    let bb5_succs: Vec<&Name> = cfg.succs(&bb5_name).sorted().collect();
    assert_eq!(bb5_succs, vec![&Name::from(14)]);

    let bb7_name = Name::from(7);
    let bb7_preds: Vec<&Name> = cfg.preds(&bb7_name).sorted().collect();
    assert_eq!(bb7_preds, vec![&Name::from(2)]);
    let bb7_succs: Vec<&Name> = cfg.succs(&bb7_name).sorted().collect();
    assert_eq!(bb7_succs, vec![&Name::from(14)]);

    let bb10_name = Name::from(10);
    let bb10_preds: Vec<&Name> = cfg.preds(&bb10_name).sorted().collect();
    assert_eq!(bb10_preds, vec![&Name::from(2)]);
    let bb10_succs: Vec<&Name> = cfg.succs(&bb10_name).sorted().collect();
    assert_eq!(bb10_succs, vec![&Name::from(14)]);

    let bb11_name = Name::from(11);
    let bb11_preds: Vec<&Name> = cfg.preds(&bb11_name).sorted().collect();
    assert_eq!(bb11_preds, vec![&Name::from(2)]);
    let bb11_succs: Vec<&Name> = cfg.succs(&bb11_name).sorted().collect();
    assert_eq!(bb11_succs, vec![&Name::from(14)]);

    let bb12_name = Name::from(12);
    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&Name::from(2)]);
    let bb12_succs: Vec<&Name> = cfg.succs(&bb12_name).sorted().collect();
    assert_eq!(bb12_succs, vec![&Name::from(14)]);

    let bb14_name = Name::from(14);
    let bb14_preds: Vec<&Name> = cfg.preds(&bb14_name).sorted().collect();
    assert_eq!(bb14_preds, vec![
        &Name::from(2),
        &Name::from(4),
        &Name::from(5),
        &Name::from(7),
        &Name::from(10),
        &Name::from(11),
        &Name::from(12),
    ]);
    let bb14_succs: Vec<&Name> = cfg.succs(&bb14_name).sorted().collect();
    assert!(bb14_succs.is_empty());
}
