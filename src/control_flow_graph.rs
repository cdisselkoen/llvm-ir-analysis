use llvm_ir::{Function, Name, Terminator};
use petgraph::prelude::{DiGraphMap, Direction};

/// The control flow graph for a particular function
pub struct ControlFlowGraph<'m> {
    /// The graph itself. Nodes are basic block names, and an edge from bbX to
    /// bbY indicates that control may (immediately) flow from bbX to bbY
    pub(crate) graph: DiGraphMap<&'m Name, ()>,

    /// Name of the entry node
    entry_node: &'m Name,
}

impl<'m> ControlFlowGraph<'m> {
    pub(crate) fn new(function: &'m Function) -> Self {
        let mut graph: DiGraphMap<&'m Name, ()> = DiGraphMap::with_capacity(
            function.basic_blocks.len(),
            2 * function.basic_blocks.len(), // arbitrary guess
        );

        for bb in &function.basic_blocks {
            match &bb.term {
                Terminator::Br(br) => {
                    graph.add_edge(&bb.name, &br.dest, ());
                },
                Terminator::CondBr(condbr) => {
                    graph.add_edge(&bb.name, &condbr.true_dest, ());
                    graph.add_edge(&bb.name, &condbr.false_dest, ());
                },
                Terminator::IndirectBr(ibr) => {
                    for dest in &ibr.possible_dests {
                        graph.add_edge(&bb.name, dest, ());
                    }
                },
                Terminator::Switch(switch) => {
                    graph.add_edge(&bb.name, &switch.default_dest, ());
                    for (_, dest) in &switch.dests {
                        graph.add_edge(&bb.name, dest, ());
                    }
                },
                Terminator::Invoke(invoke) => {
                    graph.add_edge(&bb.name, &invoke.return_label, ());
                    graph.add_edge(&bb.name, &invoke.exception_label, ());
                },
                Terminator::CleanupRet(cleanupret) => {
                    if let Some(dest) = &cleanupret.unwind_dest {
                        graph.add_edge(&bb.name, dest, ());
                    }
                },
                Terminator::CatchRet(catchret) => {
                    graph.add_edge(&bb.name, &catchret.successor, ());
                },
                Terminator::CatchSwitch(catchswitch) => {
                    if let Some(dest) = &catchswitch.default_unwind_dest {
                        graph.add_edge(&bb.name, dest, ());
                    }
                    for handler in &catchswitch.catch_handlers {
                        graph.add_edge(&bb.name, handler, ());
                    }
                },
                Terminator::CallBr(_) => unimplemented!("CallBr instruction"),
                Terminator::Ret(_)
                | Terminator::Resume(_)
                | Terminator::Unreachable(_) => {
                    // no successors from these terminators
                }
            }
        }

        Self {
            graph,
            entry_node: &function.basic_blocks[0].name,
        }
    }

    /// Get the predecessors of the basic block with the given `Name`
    pub fn preds<'s>(&'s self, block: &'m Name) -> impl Iterator<Item = &'m Name> + 's {
        self.graph.neighbors_directed(block, Direction::Incoming)
    }

    /// Get the successors of the basic block with the given `Name`
    pub fn succs<'s>(&'s self, block: &'m Name) -> impl Iterator<Item = &'m Name> + 's {
        self.graph.neighbors_directed(block, Direction::Outgoing)
    }

    /// Get the `Name` of the entry block for the function
    pub fn entry(&self) -> &'m Name {
        self.entry_node
    }
}
