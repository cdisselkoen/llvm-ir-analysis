use itertools::Itertools;
use llvm_ir::Module;
use llvm_ir_analysis::*;

fn init_logging() {
    // capture log messages with test harness
    let _ = env_logger::builder().is_test(true).try_init();
}

const HAYBALE_CALL_BC_PATH: &'static str = "../haybale/tests/bcfiles/call.bc";

#[test]
fn call_graph() {
    init_logging();
    let module = Module::from_bc_path(HAYBALE_CALL_BC_PATH)
        .unwrap_or_else(|e| panic!("Failed to parse module: {}", e));
    let analysis = Analysis::new(&module);
    let callgraph = analysis.call_graph();

    let callers: Vec<&str> = callgraph.callers("simple_callee").sorted().collect();
    assert_eq!(callers, vec![
        "caller_with_loop",
        "conditional_caller",
        "recursive_and_normal_caller",
        "simple_caller",
        "twice_caller",
    ]);
    let callees: Vec<&str> = callgraph.callees("simple_callee").sorted().collect();
    assert!(callees.is_empty());

    let callers: Vec<&str> = callgraph.callers("simple_caller").sorted().collect();
    assert_eq!(callers, vec!["nested_caller"]);
    let callees: Vec<&str> = callgraph.callees("simple_caller").sorted().collect();
    assert_eq!(callees, vec!["simple_callee"]);

    let callers: Vec<&str> = callgraph.callers("conditional_caller").sorted().collect();
    assert!(callers.is_empty());
    let callees: Vec<&str> = callgraph.callees("conditional_caller").sorted().collect();
    assert_eq!(callees, vec!["simple_callee"]);

    let callers: Vec<&str> = callgraph.callers("twice_caller").sorted().collect();
    assert!(callers.is_empty());
    let callees: Vec<&str> = callgraph.callees("twice_caller").sorted().collect();
    assert_eq!(callees, vec!["simple_callee"]);

    let callers: Vec<&str> = callgraph.callers("nested_caller").sorted().collect();
    assert!(callers.is_empty());
    let callees: Vec<&str> = callgraph.callees("nested_caller").sorted().collect();
    assert_eq!(callees, vec!["simple_caller"]);

    let callers: Vec<&str> = callgraph.callers("callee_with_loop").sorted().collect();
    assert_eq!(callers, vec!["caller_of_loop"]);
    let callees: Vec<&str> = callgraph.callees("callee_with_loop").sorted().collect();
    assert_eq!(callees, vec!["llvm.lifetime.end.p0i8", "llvm.lifetime.start.p0i8"]);

    let callers: Vec<&str> = callgraph.callers("caller_of_loop").sorted().collect();
    assert!(callers.is_empty());
    let callees: Vec<&str> = callgraph.callees("caller_of_loop").sorted().collect();
    assert_eq!(callees, vec!["callee_with_loop"]);

    let callers: Vec<&str> = callgraph.callers("caller_with_loop").sorted().collect();
    assert!(callers.is_empty());
    let callees: Vec<&str> = callgraph.callees("caller_with_loop").sorted().collect();
    assert_eq!(callees, vec!["llvm.lifetime.end.p0i8", "llvm.lifetime.start.p0i8", "simple_callee"]);

    let callers: Vec<&str> = callgraph.callers("recursive_simple").sorted().collect();
    assert_eq!(callers, vec!["recursive_simple"]);
    let callees: Vec<&str> = callgraph.callees("recursive_simple").sorted().collect();
    assert_eq!(callees, vec!["recursive_simple"]);

    let callers: Vec<&str> = callgraph.callers("recursive_double").sorted().collect();
    assert_eq!(callers, vec!["recursive_double"]);
    let callees: Vec<&str> = callgraph.callees("recursive_double").sorted().collect();
    assert_eq!(callees, vec!["recursive_double"]);

    let callers: Vec<&str> = callgraph.callers("recursive_and_normal_caller").sorted().collect();
    assert_eq!(callers, vec!["recursive_and_normal_caller"]);
    let callees: Vec<&str> = callgraph.callees("recursive_and_normal_caller").sorted().collect();
    assert_eq!(callees, vec!["recursive_and_normal_caller", "simple_callee"]);

    let callers: Vec<&str> = callgraph.callers("mutually_recursive_a").sorted().collect();
    assert_eq!(callers, vec!["mutually_recursive_b"]);
    let callees: Vec<&str> = callgraph.callees("mutually_recursive_a").sorted().collect();
    assert_eq!(callees, vec!["mutually_recursive_b"]);

    let callers: Vec<&str> = callgraph.callers("mutually_recursive_b").sorted().collect();
    assert_eq!(callers, vec!["mutually_recursive_a"]);
    let callees: Vec<&str> = callgraph.callees("mutually_recursive_b").sorted().collect();
    assert_eq!(callees, vec!["mutually_recursive_a"]);
}
