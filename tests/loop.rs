use itertools::Itertools;
use llvm_ir::{Module, Name};
use llvm_ir_analysis::*;

fn init_logging() {
    // capture log messages with test harness
    let _ = env_logger::builder().is_test(true).try_init();
}

const HAYBALE_LOOP_BC_PATH: &'static str = "../haybale/tests/bcfiles/loop.bc";

#[test]
fn while_loop_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("while_loop");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(6)]);

    let bb6_name = Name::from(6);
    let bb6_preds: Vec<&Name> = cfg.preds(&bb6_name).sorted().collect();
    assert_eq!(bb6_preds, vec![&Name::from(1), &Name::from(6)]);
    let bb6_succs: Vec<&Name> = cfg.succs(&bb6_name).sorted().collect();
    assert_eq!(bb6_succs, vec![&Name::from(6), &Name::from(12)]);

    let bb12_name = Name::from(12);
    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&Name::from(6)]);
    let bb12_succs: Vec<&Name> = cfg.succs(&bb12_name).sorted().collect();
    assert!(bb12_succs.is_empty());
}

#[test]
fn for_loop_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("for_loop");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(6), &Name::from(9)]);

    let bb6_name = Name::from(6);
    let bb6_preds: Vec<&Name> = cfg.preds(&bb6_name).sorted().collect();
    assert_eq!(bb6_preds, vec![&Name::from(1), &Name::from(9)]);
    let bb6_succs: Vec<&Name> = cfg.succs(&bb6_name).sorted().collect();
    assert!(bb6_succs.is_empty());

    let bb9_name = Name::from(9);
    let bb9_preds: Vec<&Name> = cfg.preds(&bb9_name).sorted().collect();
    assert_eq!(bb9_preds, vec![&Name::from(1), &Name::from(9)]);
    let bb9_succs: Vec<&Name> = cfg.succs(&bb9_name).sorted().collect();
    assert_eq!(bb9_succs, vec![&Name::from(6), &Name::from(9)]);
}


#[test]
fn loop_zero_iterations_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("loop_zero_iterations");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(5), &Name::from(18)]);

    let bb5_name = Name::from(5);
    let bb5_preds: Vec<&Name> = cfg.preds(&bb5_name).sorted().collect();
    assert_eq!(bb5_preds, vec![&Name::from(1)]);
    let bb5_succs: Vec<&Name> = cfg.succs(&bb5_name).sorted().collect();
    assert_eq!(bb5_succs, vec![&Name::from(8), &Name::from(11)]);

    let bb8_name = Name::from(8);
    let bb8_preds: Vec<&Name> = cfg.preds(&bb8_name).sorted().collect();
    assert_eq!(bb8_preds, vec![&Name::from(5), &Name::from(11)]);
    let bb8_succs: Vec<&Name> = cfg.succs(&bb8_name).sorted().collect();
    assert_eq!(bb8_succs, vec![&Name::from(18)]);

    let bb11_name = Name::from(11);
    let bb11_preds: Vec<&Name> = cfg.preds(&bb11_name).sorted().collect();
    assert_eq!(bb11_preds, vec![&Name::from(5), &Name::from(11)]);
    let bb11_succs: Vec<&Name> = cfg.succs(&bb11_name).sorted().collect();
    assert_eq!(bb11_succs, vec![&Name::from(8), &Name::from(11)]);

    let bb18_name = Name::from(18);
    let bb18_preds: Vec<&Name> = cfg.preds(&bb18_name).sorted().collect();
    assert_eq!(bb18_preds, vec![&Name::from(1), &Name::from(8)]);
    let bb18_succs: Vec<&Name> = cfg.succs(&bb18_name).sorted().collect();
    assert!(bb18_succs.is_empty());
}

#[test]
fn loop_with_cond_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("loop_with_cond");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(6)]);

    let bb6_name = Name::from(6);
    let bb6_preds: Vec<&Name> = cfg.preds(&bb6_name).sorted().collect();
    assert_eq!(bb6_preds, vec![&Name::from(1), &Name::from(16)]);
    let bb6_succs: Vec<&Name> = cfg.succs(&bb6_name).sorted().collect();
    assert_eq!(bb6_succs, vec![&Name::from(10), &Name::from(13)]);

    let bb10_name = Name::from(10);
    let bb10_preds: Vec<&Name> = cfg.preds(&bb10_name).sorted().collect();
    assert_eq!(bb10_preds, vec![&Name::from(6)]);
    let bb10_succs: Vec<&Name> = cfg.succs(&bb10_name).sorted().collect();
    assert_eq!(bb10_succs, vec![&Name::from(13), &Name::from(16)]);

    let bb13_name = Name::from(13);
    let bb13_preds: Vec<&Name> = cfg.preds(&bb13_name).sorted().collect();
    assert_eq!(bb13_preds, vec![&Name::from(6), &Name::from(10)]);
    let bb13_succs: Vec<&Name> = cfg.succs(&bb13_name).sorted().collect();
    assert_eq!(bb13_succs, vec![&Name::from(16)]);

    let bb16_name = Name::from(16);
    let bb16_preds: Vec<&Name> = cfg.preds(&bb16_name).sorted().collect();
    assert_eq!(bb16_preds, vec![&Name::from(10), &Name::from(13)]);
    let bb16_succs: Vec<&Name> = cfg.succs(&bb16_name).sorted().collect();
    assert_eq!(bb16_succs, vec![&Name::from(6), &Name::from(20)]);

    let bb20_name = Name::from(20);
    let bb20_preds: Vec<&Name> = cfg.preds(&bb20_name).sorted().collect();
    assert_eq!(bb20_preds, vec![&Name::from(16)]);
    let bb20_succs: Vec<&Name> = cfg.succs(&bb20_name).sorted().collect();
    assert!(bb20_succs.is_empty());
}


#[test]
fn loop_inside_cond_cfg() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("loop_inside_cond");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(5), &Name::from(11)]);

    let bb5_name = Name::from(5);
    let bb5_preds: Vec<&Name> = cfg.preds(&bb5_name).sorted().collect();
    assert_eq!(bb5_preds, vec![&Name::from(1), &Name::from(5)]);
    let bb5_succs: Vec<&Name> = cfg.succs(&bb5_name).sorted().collect();
    assert_eq!(bb5_succs, vec![&Name::from(5), &Name::from(12)]);

    let bb11_name = Name::from(11);
    let bb11_preds: Vec<&Name> = cfg.preds(&bb11_name).sorted().collect();
    assert_eq!(bb11_preds, vec![&Name::from(1)]);
    let bb11_succs: Vec<&Name> = cfg.succs(&bb11_name).sorted().collect();
    assert_eq!(bb11_succs, vec![&Name::from(12)]);

    let bb12_name = Name::from(12);
    let bb12_preds: Vec<&Name> = cfg.preds(&bb12_name).sorted().collect();
    assert_eq!(bb12_preds, vec![&Name::from(5), &Name::from(11)]);
    let bb12_succs: Vec<&Name> = cfg.succs(&bb12_name).sorted().collect();
    assert!(bb12_succs.is_empty());
}

#[test]
fn search_array() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("search_array");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(4)]);

    let bb4_name = Name::from(4);
    let bb4_preds: Vec<&Name> = cfg.preds(&bb4_name).sorted().collect();
    assert_eq!(bb4_preds, vec![&Name::from(1), &Name::from(4)]);
    let bb4_succs: Vec<&Name> = cfg.succs(&bb4_name).sorted().collect();
    assert_eq!(bb4_succs, vec![&Name::from(4), &Name::from(11)]);

    let bb11_name = Name::from(11);
    let bb11_preds: Vec<&Name> = cfg.preds(&bb11_name).sorted().collect();
    assert_eq!(bb11_preds, vec![&Name::from(4), &Name::from(16)]);
    let bb11_succs: Vec<&Name> = cfg.succs(&bb11_name).sorted().collect();
    assert_eq!(bb11_succs, vec![&Name::from(16), &Name::from(19)]);

    let bb16_name = Name::from(16);
    let bb16_preds: Vec<&Name> = cfg.preds(&bb16_name).sorted().collect();
    assert_eq!(bb16_preds, vec![&Name::from(11)]);
    let bb16_succs: Vec<&Name> = cfg.succs(&bb16_name).sorted().collect();
    assert_eq!(bb16_succs, vec![&Name::from(11), &Name::from(21)]);

    let bb19_name = Name::from(19);
    let bb19_preds: Vec<&Name> = cfg.preds(&bb19_name).sorted().collect();
    assert_eq!(bb19_preds, vec![&Name::from(11)]);
    let bb19_succs: Vec<&Name> = cfg.succs(&bb19_name).sorted().collect();
    assert_eq!(bb19_succs, vec![&Name::from(21)]);

    let bb21_name = Name::from(21);
    let bb21_preds: Vec<&Name> = cfg.preds(&bb21_name).sorted().collect();
    assert_eq!(bb21_preds, vec![&Name::from(16), &Name::from(19)]);
    let bb21_succs: Vec<&Name> = cfg.succs(&bb21_name).sorted().collect();
    assert!(bb21_succs.is_empty());
}

#[test]
fn nested_loop() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_LOOP_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let cfg = analysis.control_flow_graph("nested_loop");

    let bb1_name = Name::from(1);
    let bb1_preds: Vec<&Name> = cfg.preds(&bb1_name).sorted().collect();
    assert!(bb1_preds.is_empty());
    let bb1_succs: Vec<&Name> = cfg.succs(&bb1_name).sorted().collect();
    assert_eq!(bb1_succs, vec![&Name::from(5), &Name::from(7)]);

    let bb5_name = Name::from(5);
    let bb5_preds: Vec<&Name> = cfg.preds(&bb5_name).sorted().collect();
    assert_eq!(bb5_preds, vec![&Name::from(1), &Name::from(10)]);
    let bb5_succs: Vec<&Name> = cfg.succs(&bb5_name).sorted().collect();
    assert_eq!(bb5_succs, vec![&Name::from(13)]);

    let bb7_name = Name::from(7);
    let bb7_preds: Vec<&Name> = cfg.preds(&bb7_name).sorted().collect();
    assert_eq!(bb7_preds, vec![&Name::from(1), &Name::from(10)]);
    let bb7_succs: Vec<&Name> = cfg.succs(&bb7_name).sorted().collect();
    assert!(bb7_succs.is_empty());

    let bb10_name = Name::from(10);
    let bb10_preds: Vec<&Name> = cfg.preds(&bb10_name).sorted().collect();
    assert_eq!(bb10_preds, vec![&Name::from(13)]);
    let bb10_succs: Vec<&Name> = cfg.succs(&bb10_name).sorted().collect();
    assert_eq!(bb10_succs, vec![&Name::from(5), &Name::from(7)]);

    let bb13_name = Name::from(13);
    let bb13_preds: Vec<&Name> = cfg.preds(&bb13_name).sorted().collect();
    assert_eq!(bb13_preds, vec![&Name::from(5), &Name::from(13)]);
    let bb13_succs: Vec<&Name> = cfg.succs(&bb13_name).sorted().collect();
    assert_eq!(bb13_succs, vec![&Name::from(10), &Name::from(13)]);
}
