use crate::control_flow_graph::ControlFlowGraph;
use llvm_ir::Name;
use log::debug;
use petgraph::prelude::{Dfs, DiGraphMap, Direction};
use petgraph::visit::Walker;
use std::cmp::Ordering;
use std::collections::HashMap;

/// The dominator tree for a particular function
pub struct DominatorTree<'m> {
    /// The graph itself. Nodes are basic block names, and an edge from bbX to
    /// bbY indicates that bbX is the immediate dominator of bbY.
    ///
    /// That is:
    ///   - bbX strictly dominates bbY, i.e., bbX appears on every control-flow
    ///     path from the entry block to bbY (but bbX =/= bbY)
    ///   - Of the blocks that strictly dominate bbY, bbX is the closest to bbY
    ///     (farthest from entry) along paths from the entry block to bbY
    graph: DiGraphMap<&'m Name, ()>,

    /// Name of the entry node
    entry_node: &'m Name,
}

/// Contains state used when constructing the `DominatorTree`
struct DomTreeBuilder<'m, 'a> {
    /// The `ControlFlowGraph` we're working from
    cfg: &'a ControlFlowGraph<'m>,

    /// Map from block name to its rpo number.
    ///
    /// Unreachable blocks won't be in this map; all reachable blocks will have
    /// positive rpo numbers.
    rpo_numbers: HashMap<&'m Name, usize>,

    /// Map from block name to the current estimate for its immediate dominator
    /// (or `None` for the entry block).
    ///
    /// Unreachable blocks won't be in this map.
    idoms: HashMap<&'m Name, Option<&'m Name>>,
}

impl<'m, 'a> DomTreeBuilder<'m, 'a> {
    /// Construct a new `DomTreeBuilder`.
    ///
    /// This will have no estimates for the immediate dominators.
    fn new(cfg: &'a ControlFlowGraph<'m>) -> Self {
        Self {
            cfg,
            rpo_numbers: Dfs::new(&cfg.graph, cfg.entry())
                .iter(&cfg.graph)
                .zip(1..)
                .collect(),
            idoms: HashMap::new(),
        }
    }

    /// Build the dominator tree
    fn build(mut self) -> DiGraphMap<&'m Name, ()> {
        // algorithm heavily inspired by the domtree algorithm in Cranelift,
        // which itself is Keith D. Cooper's "Simple, Fast, Dominator Algorithm"
        // according to comments in Cranelift's code.

        // first compute initial (preliminary) estimates for the immediate
        // dominator of each block
        for block in Dfs::new(&self.cfg.graph, self.cfg.entry()).iter(&self.cfg.graph) {
            self.idoms.insert(block, self.compute_idom(block));
        }

        let mut changed = true;
        while changed {
            changed = false;
            for block in Dfs::new(&self.cfg.graph, self.cfg.entry()).iter(&self.cfg.graph) {
                let idom = self.compute_idom(block);
                let prev_idom = self.idoms.get_mut(block).expect("All nodes in the dfs should have an initialized idom by now");
                if idom != *prev_idom {
                    *prev_idom = idom;
                    changed = true;
                }
            }
        }

        DiGraphMap::from_edges(
            self.idoms.into_iter().filter_map(|(block, idom)| Some((idom?, block)))
        )
    }

    /// Compute the immediate dominator for `block` using the current `idom`
    /// states for the nodes.
    ///
    /// `block` must be reachable in the CFG. Returns `None` only for the entry
    /// block.
    fn compute_idom(&self, block: &'m Name) -> Option<&'m Name> {
        debug!("domtree: compute_idom for {}", block);
        if block == self.cfg.entry() {
            return None;
        }
        // technically speaking, these are just the reachable preds which already have an idom estimate
        let mut reachable_preds = self.cfg
            .preds(block)
            .filter(|block| self.idoms.contains_key(block));

        let mut idom = reachable_preds
            .next()
            .expect("expected a reachable block to have at least one reachable predecessor");

        for pred in reachable_preds {
            idom = self.common_dominator(idom, pred);
        }

        Some(idom)
    }

    /// Compute the common dominator of two basic blocks.
    ///
    /// Both blocks are assumed to be reachable.
    fn common_dominator(
        &self,
        mut block_a: &'m Name,
        mut block_b: &'m Name,
    ) -> &'m Name {
        loop {
            match self.rpo_numbers[block_a].cmp(&self.rpo_numbers[block_b]) {
                Ordering::Less => {
                    block_b = self.idoms[block_b].unwrap_or(self.cfg.entry());
                },
                Ordering::Greater => {
                    block_a = self.idoms[block_a].unwrap_or(self.cfg.entry());
                },
                Ordering::Equal => break,
            }
        }

        block_a
    }
}

impl<'m> DominatorTree<'m> {
    pub(crate) fn new(cfg: &ControlFlowGraph<'m>) -> Self {
        Self {
            graph: DomTreeBuilder::new(cfg).build(),
            entry_node: cfg.entry(),
        }
    }

    /// Get the immediate dominator of the basic block with the given `Name`.
    ///
    /// This will be `None` for the entry block or for any unreachable blocks,
    /// and `Some` for all other blocks.
    ///
    /// A block bbX is the immediate dominator of bbY if and only if:
    ///   - bbX strictly dominates bbY, i.e., bbX appears on every control-flow
    ///     path from the entry block to bbY (but bbX =/= bbY)
    ///   - Of the blocks that strictly dominate bbY, bbX is the closest to bbY
    ///     (farthest from entry) along paths from the entry block to bbY
    pub fn idom(&self, block: &'m Name) -> Option<&'m Name> {
        let mut parents = self.graph.neighbors_directed(block, Direction::Incoming);
        let idom = parents.next();
        if let Some(_) = parents.next() {
            panic!("Block {:?} should have only one immediate dominator");
        }
        idom
    }

    /// Get the children of the given basic block in the dominator tree, i.e.,
    /// get all the blocks which are immediately dominated by `block`.
    ///
    /// See notes on `idom()`.
    pub fn children<'s>(&'s self, block: &'m Name) -> impl Iterator<Item = &'m Name> + 's {
        self.graph.neighbors_directed(block, Direction::Outgoing)
    }

    /// Get the `Name` of the entry block for the function
    pub fn entry(&self) -> &'m Name {
        self.entry_node
    }
}
