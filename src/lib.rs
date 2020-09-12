mod call_graph;
mod functions_by_type;

pub use crate::call_graph::CallGraph;
pub use crate::functions_by_type::FunctionsByType;
use llvm_ir::Module;
use std::cell::{RefMut, RefCell};

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
        }
    }

    /// Get the `CallGraph` for the `Module`
    pub fn call_graph(&self) -> RefMut<CallGraph<'m>> {
        let functions_by_type = self.functions_by_type(); // for the borrow checker, so we can use `functions_by_type` below without borrowing `self`
        let refmut = self.call_graph.borrow_mut();
        RefMut::map(refmut, |o| o.get_or_insert_with(|| {
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
}
