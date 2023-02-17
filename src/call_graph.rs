use crate::functions_by_type::FunctionsByType;
use either::Either;
use llvm_ir::{Constant, Instruction, Module, Name, Operand, Type};
use petgraph::prelude::*;

/// The call graph for the analyzed `Module`(s): which functions may call which
/// other functions.
///
/// To construct a `CallGraph`, use [`ModuleAnalysis`](struct.ModuleAnalysis.html)
/// or [`CrossModuleAnalysis`](struct.CrossModuleAnalysis.html).
pub struct CallGraph<'m> {
    /// the call graph itself. Nodes are function names, and an edge from F to G
    /// indicates F may call G
    graph: DiGraphMap<&'m str, ()>,
}

impl<'m> CallGraph<'m> {
    pub(crate) fn new(
        modules: impl IntoIterator<Item = &'m Module>,
        functions_by_type: &FunctionsByType<'m>,
    ) -> Self {
        let mut graph: DiGraphMap<&'m str, ()> = DiGraphMap::new();

        // Find all call instructions and add the appropriate edges
        for module in modules {
            for f in &module.functions {
                graph.add_node(&f.name); // just to ensure all functions end up getting nodes in the graph by the end
                for bb in &f.basic_blocks {
                    for inst in &bb.instrs {
                        if let Instruction::Call(call) = inst {
                            match &call.function {
                                Either::Right(Operand::ConstantOperand(cref)) => {
                                    match cref.as_ref() {
                                        #[cfg(not(feature = "llvm-15"))]
                                        Constant::GlobalReference { name: Name::Name(name), .. } => {
                                            graph.add_edge(&f.name, name, ());
                                        },
                                        #[cfg(feature = "llvm-15")]
                                        Constant::GlobalReference { name: name, .. } => {
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
                                                #[cfg(not(feature = "llvm-15"))]
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
                                        #[cfg(not(feature = "llvm-15"))]
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
        }

        Self {
            graph,
        }
    }

    /// Get the names of functions in the analyzed `Module`(s) which may call the
    /// given function.
    ///
    /// This analysis conservatively assumes that function pointers may point to
    /// any function in the analyzed `Module`(s) that has the appropriate type.
    ///
    /// Panics if the given function is not found in the analyzed `Module`(s).
    pub fn callers<'s>(&'s self, func_name: &'m str) -> impl Iterator<Item = &'m str> + 's {
        if !self.graph.contains_node(func_name) {
            panic!("callers(): function named {:?} not found in the Module(s)", func_name)
        }
        self.graph.neighbors_directed(func_name, Direction::Incoming)
    }

    /// Get the names of functions in the analyzed `Module`(s) which may be
    /// called by the given function.
    ///
    /// This analysis conservatively assumes that function pointers may point to
    /// any function in the analyzed `Module`(s) that has the appropriate type.
    ///
    /// Panics if the given function is not found in the analyzed `Module`(s).
    pub fn callees<'s>(&'s self, func_name: &'m str) -> impl Iterator<Item = &'m str> + 's {
        if !self.graph.contains_node(func_name) {
            panic!("callees(): function named {:?} not found in the Module(s)", func_name)
        }
        self.graph.neighbors_directed(func_name, Direction::Outgoing)
    }
}
