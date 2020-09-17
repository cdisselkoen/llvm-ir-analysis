use crate::functions_by_type::FunctionsByType;
use either::Either;
use llvm_ir::{Constant, Instruction, Module, Name, Operand, Type};
use petgraph::prelude::*;

/// The call graph for the `Module`: which functions may call which other
/// functions.
///
/// To construct a `CallGraph`, use [`Analysis`](struct.Analysis.html).
pub struct CallGraph<'m> {
    /// the call graph itself. Nodes are function names, and an edge from F to G
    /// indicates F may call G
    graph: DiGraphMap<&'m str, ()>,
}

impl<'m> CallGraph<'m> {
    pub(crate) fn new(module: &'m Module, functions_by_type: &FunctionsByType<'m>) -> Self {
        let mut graph: DiGraphMap<&'m str, ()> = DiGraphMap::with_capacity(
            module.functions.len(),
            3 * module.functions.len(), // arbitrary guess
        );

        // Add all function names as nodes in the graph
        for f in &module.functions {
            graph.add_node(&f.name);
        }

        // Find all call instructions and add the appropriate edges
        for f in &module.functions {
            for bb in &f.basic_blocks {
                for inst in &bb.instrs {
                    if let Instruction::Call(call) = inst {
                        match &call.function {
                            Either::Right(Operand::ConstantOperand(cref)) => {
                                match cref.as_ref() {
                                    Constant::GlobalReference { name: Name::Name(name), .. } => {
                                        graph.add_edge(&f.name, name, ());
                                    },
                                    Constant::GlobalReference { name, .. } => {
                                        unimplemented!("Call of a function with a numbered name: {:?}", name)
                                    },
                                    _ => {
                                        // a constant function pointer.
                                        // Assume that this function pointer could point
                                        // to any function in the current module that has
                                        // the appropriate type
                                        let func_ty = match module.type_of(&call.function).as_ref() {
                                            Type::PointerType { pointee_type, .. } => pointee_type.clone(),
                                            ty => panic!("Expected function pointer to have pointer type, but got {:?}", ty),
                                        };
                                        for target in functions_by_type.functions_with_type(&func_ty) {
                                            graph.add_edge(&f.name, target, ());
                                        }
                                    },
                                }
                            },
                            Either::Right(_) => {
                                // Assume that this function pointer could point to any
                                // function in the current module that has the
                                // appropriate type
                                let func_ty = match module.type_of(&call.function).as_ref() {
                                    Type::PointerType { pointee_type, .. } => pointee_type.clone(),
                                    ty => panic!("Expected function pointer to have pointer type, but got {:?}", ty),
                                };
                                for target in functions_by_type.functions_with_type(&func_ty) {
                                    graph.add_edge(&f.name, target, ());
                                }
                            },
                            Either::Left(_) => {}, // ignore calls to inline assembly
                        }
                    }
                }
            }
        }

        Self {
            graph,
        }
    }

    /// Get the names of functions in this `Module` which may call the given
    /// function.
    ///
    /// This analysis conservatively assumes that function pointers may point to
    /// any function in the `Module` that has the appropriate type.
    ///
    /// Panics if the given function is not found in the `Module`.
    pub fn callers<'s>(&'s self, func_name: &'m str) -> impl Iterator<Item = &'m str> + 's {
        if !self.graph.contains_node(func_name) {
            panic!("callers(): function named {:?} not found in the Module", func_name)
        }
        self.graph.neighbors_directed(func_name, Direction::Incoming)
    }

    /// Get the names of functions in this `Module` which may be called by the
    /// given function.
    ///
    /// This analysis conservatively assumes that function pointers may point to
    /// any function in the `Module` that has the appropriate type.
    ///
    /// Panics if the given function is not found in the `Module`.
    pub fn callees<'s>(&'s self, func_name: &'m str) -> impl Iterator<Item = &'m str> + 's {
        if !self.graph.contains_node(func_name) {
            panic!("callees(): function named {:?} not found in the Module", func_name)
        }
        self.graph.neighbors_directed(func_name, Direction::Outgoing)
    }
}
