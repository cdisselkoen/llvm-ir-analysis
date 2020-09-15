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
        let entry = cfg.entry();
        assert_eq!(cfg.preds(entry).count(), 0);
        let succs = cfg.succs(entry).collect::<Vec<_>>();
        assert_eq!(succs, vec![CFGNode::Return]);
    }
}

#[test]
fn conditional_true_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("conditional_true");

    // CFG:
    //     2
    //   /   \
    //  4     8
    //   \   /
    //    12

    let bb2_name = Name::from(2);
    let _bb2_node = CFGNode::Block(&bb2_name);
    let bb4_name = Name::from(4);
    let bb4_node = CFGNode::Block(&bb4_name);
    let bb8_name = Name::from(8);
    let bb8_node = CFGNode::Block(&bb8_name);
    let bb12_name = Name::from(12);
    let bb12_node = CFGNode::Block(&bb12_name);

    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<CFGNode> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![bb4_node, bb8_node]);

    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&bb2_name]);
    let bb4_succs: Vec<CFGNode> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![bb12_node]);

    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&bb2_name]);
    let bb8_succs: Vec<CFGNode> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![bb12_node]);

    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&bb4_name, &bb8_name]);
    let bb12_succs: Vec<CFGNode> = cfg.succs(&bb12_name).sorted().collect();
    assert_eq!(bb12_succs, vec![CFGNode::Return]);
}

#[test]
fn conditional_false_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("conditional_false");

    // CFG:
    //     2
    //   /   \
    //  4     8
    //   \   /
    //    12

    let bb2_name = Name::from(2);
    let _bb2_node = CFGNode::Block(&bb2_name);
    let bb4_name = Name::from(4);
    let bb4_node = CFGNode::Block(&bb4_name);
    let bb8_name = Name::from(8);
    let bb8_node = CFGNode::Block(&bb8_name);
    let bb12_name = Name::from(12);
    let bb12_node = CFGNode::Block(&bb12_name);

    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<CFGNode> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![bb4_node, bb8_node]);

    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&bb2_name]);
    let bb4_succs: Vec<CFGNode> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![bb12_node]);

    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&bb2_name]);
    let bb8_succs: Vec<CFGNode> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![bb12_node]);

    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&bb4_name, &bb8_name]);
    let bb12_succs: Vec<CFGNode> = cfg.succs(&bb12_name).sorted().collect();
    assert_eq!(bb12_succs, vec![CFGNode::Return]);
}

#[test]
fn conditional_nozero_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("conditional_nozero");

    // CFG:
    //  2
    //  | \
    //  |  4
    //  |  | \
    //  |  |  8
    //  |  6  | \
    //  |  |  10 12
    //  |  |  |  |
    //  |  |  | /
    //   \ | / /
    //     14

    let bb2_name = Name::from(2);
    let _bb2_node = CFGNode::Block(&bb2_name);
    let bb4_name = Name::from(4);
    let bb4_node = CFGNode::Block(&bb4_name);
    let bb6_name = Name::from(6);
    let bb6_node = CFGNode::Block(&bb6_name);
    let bb8_name = Name::from(8);
    let bb8_node = CFGNode::Block(&bb8_name);
    let bb10_name = Name::from(10);
    let bb10_node = CFGNode::Block(&bb10_name);
    let bb12_name = Name::from(12);
    let bb12_node = CFGNode::Block(&bb12_name);
    let bb14_name = Name::from(14);
    let bb14_node = CFGNode::Block(&bb14_name);

    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<CFGNode> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![bb4_node, bb14_node]);

    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&bb2_name]);
    let bb4_succs: Vec<CFGNode> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![bb6_node, bb8_node]);

    let bb6_preds: Vec<&Name> = cfg.preds(&bb6_name).sorted().collect();
    assert_eq!(bb6_preds, vec![&bb4_name]);
    let bb6_succs: Vec<CFGNode> = cfg.succs(&bb6_name).sorted().collect();
    assert_eq!(bb6_succs, vec![bb14_node]);

    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&bb4_name]);
    let bb8_succs: Vec<CFGNode> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![bb10_node, bb12_node]);

    let bb10_preds: Vec<&Name> = cfg.preds(&bb10_name).sorted().collect();
    assert_eq!(bb10_preds, vec![&bb8_name]);
    let bb10_succs: Vec<CFGNode> = cfg.succs(&bb10_name).sorted().collect();
    assert_eq!(bb10_succs, vec![bb14_node]);

    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&bb8_name]);
    let bb12_succs: Vec<CFGNode> = cfg.succs(&bb12_name).sorted().collect();
    assert_eq!(bb12_succs, vec![bb14_node]);

    let bb14_preds: Vec<&Name> = cfg.preds(&bb14_name).sorted().collect();
    assert_eq!(bb14_preds, vec![&bb2_name, &bb6_name, &bb10_name, &bb12_name]);
    let bb14_succs: Vec<CFGNode> = cfg.succs(&bb14_name).sorted().collect();
    assert_eq!(bb14_succs, vec![CFGNode::Return]);
}

#[test]
fn has_switch_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("has_switch");

    // CFG:
    //           2
    //     ___ / | \ ___
    //   /  / |  |  | \  \
    //  |  |  |  |  |  \  \
    //  |  |  |  |  |   |  \
    //  4  5  7  |  10  11  12
    //   \  \  \ | /   /   /
    //    \  \ _ | __ /   /
    //     \ ___ | _____ /
    //           |
    //           14

    let bb2_name = Name::from(2);
    let _bb2_node = CFGNode::Block(&bb2_name);
    let bb4_name = Name::from(4);
    let bb4_node = CFGNode::Block(&bb4_name);
    let bb5_name = Name::from(5);
    let bb5_node = CFGNode::Block(&bb5_name);
    let bb7_name = Name::from(7);
    let bb7_node = CFGNode::Block(&bb7_name);
    let bb10_name = Name::from(10);
    let bb10_node = CFGNode::Block(&bb10_name);
    let bb11_name = Name::from(11);
    let bb11_node = CFGNode::Block(&bb11_name);
    let bb12_name = Name::from(12);
    let bb12_node = CFGNode::Block(&bb12_name);
    let bb14_name = Name::from(14);
    let bb14_node = CFGNode::Block(&bb14_name);

    let bb2_preds: Vec<&Name> = cfg.preds(&bb2_name).sorted().collect();
    assert!(bb2_preds.is_empty());
    let bb2_succs: Vec<CFGNode> = cfg.succs(&bb2_name).sorted().collect();
    assert_eq!(bb2_succs, vec![
        bb4_node,
        bb5_node,
        bb7_node,
        bb10_node,
        bb11_node,
        bb12_node,
        bb14_node,
    ]);

    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&bb2_name]);
    let bb4_succs: Vec<CFGNode> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![bb14_node]);

    let bb5_preds: Vec<&Name> = cfg.preds(&bb5_name).sorted().collect();
    assert_eq!(bb5_preds, vec![&bb2_name]);
    let bb5_succs: Vec<CFGNode> = cfg.succs(&bb5_name).sorted().collect();
    assert_eq!(bb5_succs, vec![bb14_node]);

    let bb7_preds: Vec<&Name> = cfg.preds(&bb7_name).sorted().collect();
    assert_eq!(bb7_preds, vec![&bb2_name]);
    let bb7_succs: Vec<CFGNode> = cfg.succs(&bb7_name).sorted().collect();
    assert_eq!(bb7_succs, vec![bb14_node]);

    let bb10_preds: Vec<&Name> = cfg.preds(&bb10_name).sorted().collect();
    assert_eq!(bb10_preds, vec![&bb2_name]);
    let bb10_succs: Vec<CFGNode> = cfg.succs(&bb10_name).sorted().collect();
    assert_eq!(bb10_succs, vec![bb14_node]);

    let bb11_preds: Vec<&Name> = cfg.preds(&bb11_name).sorted().collect();
    assert_eq!(bb11_preds, vec![&bb2_name]);
    let bb11_succs: Vec<CFGNode> = cfg.succs(&bb11_name).sorted().collect();
    assert_eq!(bb11_succs, vec![bb14_node]);

    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&bb2_name]);
    let bb12_succs: Vec<CFGNode> = cfg.succs(&bb12_name).sorted().collect();
    assert_eq!(bb12_succs, vec![bb14_node]);

    let bb14_preds: Vec<&Name> = cfg.preds(&bb14_name).sorted().collect();
    assert_eq!(bb14_preds, vec![
        &bb2_name,
        &bb4_name,
        &bb5_name,
        &bb7_name,
        &bb10_name,
        &bb11_name,
        &bb12_name,
    ]);
    let bb14_succs: Vec<CFGNode> = cfg.succs(&bb14_name).sorted().collect();
    assert_eq!(bb14_succs, vec![CFGNode::Return]);
}

#[test]
fn trivial_domtrees() {
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
        let domtree = analysis.dominator_tree(func_name);
        let entry = domtree.entry();
        assert_eq!(domtree.idom(entry), None);
        assert_eq!(domtree.children(entry).collect::<Vec<_>>(), vec![CFGNode::Return]);

        let postdomtree = analysis.postdominator_tree(func_name);
        assert_eq!(postdomtree.ipostdom(entry), CFGNode::Return);
        assert_eq!(postdomtree.children(entry).count(), 0);
    }
}

#[test]
fn conditional_true_domtree() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);

    // CFG:
    //     2
    //   /   \
    //  4     8
    //   \   /
    //    12

    let bb2_name = Name::from(2);
    let _bb2_node = CFGNode::Block(&bb2_name);
    let bb4_name = Name::from(4);
    let bb4_node = CFGNode::Block(&bb4_name);
    let bb8_name = Name::from(8);
    let bb8_node = CFGNode::Block(&bb8_name);
    let bb12_name = Name::from(12);
    let bb12_node = CFGNode::Block(&bb12_name);

    let domtree = analysis.dominator_tree("conditional_true");

    assert_eq!(domtree.idom(&bb2_name), None);
    let children: Vec<CFGNode> = domtree.children(&bb2_name).sorted().collect();
    assert_eq!(children, vec![bb4_node, bb8_node, bb12_node]);

    assert_eq!(domtree.idom(&bb4_name), Some(&bb2_name));
    let children: Vec<CFGNode> = domtree.children(&bb4_name).sorted().collect();
    assert!(children.is_empty());

    assert_eq!(domtree.idom(&bb8_name), Some(&bb2_name));
    let children: Vec<CFGNode> = domtree.children(&bb8_name).sorted().collect();
    assert!(children.is_empty());

    assert_eq!(domtree.idom(&bb8_name), Some(&bb2_name));
    let children: Vec<CFGNode> = domtree.children(&bb12_name).sorted().collect();
    assert_eq!(children, vec![CFGNode::Return]);

    let postdomtree = analysis.postdominator_tree("conditional_true");
    assert_eq!(postdomtree.ipostdom(&bb2_name), bb12_node);
    assert_eq!(postdomtree.ipostdom(&bb4_name), bb12_node);
    assert_eq!(postdomtree.ipostdom(&bb8_name), bb12_node);
    assert_eq!(postdomtree.ipostdom(&bb12_name), CFGNode::Return);
}

#[test]
fn conditional_false_domtree() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);

    // CFG:
    //     2
    //   /   \
    //  4     8
    //   \   /
    //    12

    let bb2_name = Name::from(2);
    let _bb2_node = CFGNode::Block(&bb2_name);
    let bb4_name = Name::from(4);
    let bb4_node = CFGNode::Block(&bb4_name);
    let bb8_name = Name::from(8);
    let bb8_node = CFGNode::Block(&bb8_name);
    let bb12_name = Name::from(12);
    let bb12_node = CFGNode::Block(&bb12_name);

    let domtree = analysis.dominator_tree("conditional_false");

    assert_eq!(domtree.idom(&bb2_name), None);
    let children: Vec<CFGNode> = domtree.children(&bb2_name).sorted().collect();
    assert_eq!(children, vec![bb4_node, bb8_node, bb12_node]);

    assert_eq!(domtree.idom(&bb4_name), Some(&bb2_name));
    let children: Vec<CFGNode> = domtree.children(&bb4_name).sorted().collect();
    assert!(children.is_empty());

    assert_eq!(domtree.idom(&bb8_name), Some(&bb2_name));
    let children: Vec<CFGNode> = domtree.children(&bb8_name).sorted().collect();
    assert!(children.is_empty());

    assert_eq!(domtree.idom(&bb8_name), Some(&bb2_name));
    let children: Vec<CFGNode> = domtree.children(&bb12_name).sorted().collect();
    assert_eq!(children, vec![CFGNode::Return]);

    let postdomtree = analysis.postdominator_tree("conditional_false");
    assert_eq!(postdomtree.ipostdom(&bb2_name), bb12_node);
    assert_eq!(postdomtree.ipostdom(&bb4_name), bb12_node);
    assert_eq!(postdomtree.ipostdom(&bb8_name), bb12_node);
    assert_eq!(postdomtree.ipostdom(&bb12_name), CFGNode::Return);
}

#[test]
fn conditional_nozero_domtree() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);

    // CFG:
    //  2
    //  | \
    //  |  4
    //  |  | \
    //  |  |  8
    //  |  6  | \
    //  |  |  10 12
    //  |  |  |  |
    //  |  |  | /
    //   \ | / /
    //     14

    let domtree = analysis.dominator_tree("conditional_nozero");
    assert_eq!(domtree.idom(&Name::from(2)), None);
    assert_eq!(domtree.idom(&Name::from(4)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(6)), Some(&Name::from(4)));
    assert_eq!(domtree.idom(&Name::from(8)), Some(&Name::from(4)));
    assert_eq!(domtree.idom(&Name::from(10)), Some(&Name::from(8)));
    assert_eq!(domtree.idom(&Name::from(12)), Some(&Name::from(8)));
    assert_eq!(domtree.idom(&Name::from(14)), Some(&Name::from(2)));

    let postdomtree = analysis.postdominator_tree("conditional_nozero");
    assert_eq!(postdomtree.ipostdom(&Name::from(2)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(4)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(6)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(8)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(10)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(12)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(14)), CFGNode::Return);
}

#[test]
fn has_switch_domtree() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_BASIC_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);

    // CFG:
    //           2
    //     ___ / | \ ___
    //   /  / |  |  | \  \
    //  |  |  |  |  |  \  \
    //  |  |  |  |  |   |  \
    //  4  5  7  |  10  11  12
    //   \  \  \ | /   /   /
    //    \  \ _ | __ /   /
    //     \ ___ | _____ /
    //           |
    //           14

    let domtree = analysis.dominator_tree("has_switch");
    assert_eq!(domtree.idom(&Name::from(2)), None);
    assert_eq!(domtree.idom(&Name::from(4)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(5)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(7)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(10)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(11)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(12)), Some(&Name::from(2)));
    assert_eq!(domtree.idom(&Name::from(14)), Some(&Name::from(2)));

    let postdomtree = analysis.postdominator_tree("has_switch");
    assert_eq!(postdomtree.ipostdom(&Name::from(2)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(4)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(5)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(7)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(10)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(11)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(12)), CFGNode::Block(&Name::from(14)));
    assert_eq!(postdomtree.ipostdom(&Name::from(14)), CFGNode::Return);
}
