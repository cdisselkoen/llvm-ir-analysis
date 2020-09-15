mod call_graph;
mod control_flow_graph;
mod dominator_tree;
mod functions_by_type;

pub use crate::call_graph::CallGraph;
pub use crate::control_flow_graph::ControlFlowGraph;
pub use crate::dominator_tree::DominatorTree;
pub use crate::functions_by_type::FunctionsByType;
use llvm_ir::Module;
use std::cell::{RefMut, RefCell};
use std::collections::HashMap;

/// Computes (and caches the results of) various analyses on a given `Module`
pub struct Analysis<'m> {
    /// Reference to the `llvm-ir` `Module`
    module: &'m Module,
    /// Call graph.
    /// This is `None` if not computed yet
    call_graph: RefCell<Option<CallGraph<'m>>>,
    /// `FunctionsByType`, which allows you to iterate over functions by type.
    /// This is `None` if not computed yet
    functions_by_type: RefCell<Option<FunctionsByType<'m>>>,
    /// Map from function name to the `ControlFlowGraph` for that function.
    /// The hashmap starts empty and is populated on demand.
    control_flow_graphs: RefCell<HashMap<&'m str, ControlFlowGraph<'m>>>,
    /// Map from function name to the `DominatorTree` for that function.
    /// The hashmap starts empty and is populated on demand.
    dominator_trees: RefCell<HashMap<&'m str, DominatorTree<'m>>>,
}

impl<'m> Analysis<'m> {
    /// Create a new `Analysis` for the given `Module`.
    ///
    /// This method itself is cheap; individual analyses will be computed lazily
    /// on demand.
    pub fn new(module: &'m Module) -> Self {
        Self {
            module,
            call_graph: RefCell::new(None),
            functions_by_type: RefCell::new(None),
            control_flow_graphs: RefCell::new(HashMap::new()),
            dominator_trees: RefCell::new(HashMap::new()),
        }
    }

    /// Get the `CallGraph` for the `Module`
    pub fn call_graph(&self) -> RefMut<CallGraph<'m>> {
        let refmut = self.call_graph.borrow_mut();
        RefMut::map(refmut, |o| o.get_or_insert_with(|| {
            let functions_by_type = self.functions_by_type();
            CallGraph::new(self.module, &functions_by_type)
        }))
    }

    /// Get the `FunctionsByType` for the `Module`
    pub fn functions_by_type(&self) -> RefMut<FunctionsByType<'m>> {
        let refmut = self.functions_by_type.borrow_mut();
        RefMut::map(refmut, |o| o.get_or_insert_with(|| {
            FunctionsByType::new(self.module)
        }))
    }

    /// Get the `ControlFlowGraph` for the function with the given name
    ///
    /// Panics if no function of that name exists in the `Module`.
    pub fn control_flow_graph(&self, func_name: &'m str) -> RefMut<ControlFlowGraph<'m>> {
        let refmut = self.control_flow_graphs.borrow_mut();
        RefMut::map(refmut, |map| map.entry(func_name).or_insert_with(|| {
            let func = self.module.get_func_by_name(func_name)
                .unwrap_or_else(|| panic!("Function named {:?} not found in the Module", func_name));
            ControlFlowGraph::new(func)
        }))
    }

    /// Get the `DominatorTree` for the function with the given name
    ///
    /// Panics if no function of that name exists in the `Module`.
    pub fn dominator_tree(&self, func_name: &'m str) -> RefMut<DominatorTree<'m>> {
        let refmut = self.dominator_trees.borrow_mut();
        RefMut::map(refmut, |map| map.entry(func_name).or_insert_with( || {
            let cfg = self.control_flow_graph(func_name);
            DominatorTree::new(&cfg)
        }))
    }
}
