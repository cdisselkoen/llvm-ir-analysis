use crate::control_flow_graph::{CFGNode, ControlFlowGraph};
use llvm_ir::Name;
use log::debug;
use petgraph::prelude::{Dfs, DiGraphMap, Direction};
use petgraph::visit::Walker;
use std::cmp::Ordering;
use std::collections::HashMap;

/// The dominator tree for a particular function
pub struct DominatorTree<'m> {
    /// The graph itself. An edge from bbX to bbY indicates that bbX is the
    /// immediate dominator of bbY.
    ///
    /// That is:
    ///   - bbX strictly dominates bbY, i.e., bbX appears on every control-flow
    ///     path from the entry block to bbY (but bbX =/= bbY)
    ///   - Of the blocks that strictly dominate bbY, bbX is the closest to bbY
    ///     (farthest from entry) along paths from the entry block to bbY
    graph: DiGraphMap<CFGNode<'m>, ()>,

    /// Name of the entry node
    entry_node: &'m Name,
}

/// Contains state used when constructing the `DominatorTree`
struct DomTreeBuilder<'m, 'a> {
    /// The `ControlFlowGraph` we're working from
    cfg: &'a ControlFlowGraph<'m>,

    /// Map from `CFGNode` to its rpo number.
    ///
    /// Unreachable blocks won't be in this map; all reachable blocks will have
    /// positive rpo numbers.
    rpo_numbers: HashMap<CFGNode<'m>, usize>,

    /// Map from `CFGNode` to the current estimate for its immediate dominator
    /// (the entry node maps to `None`).
    ///
    /// Unreachable blocks won't be in this map.
    idoms: HashMap<CFGNode<'m>, Option<&'m Name>>,
}

impl<'m, 'a> DomTreeBuilder<'m, 'a> {
    /// Construct a new `DomTreeBuilder`.
    ///
    /// This will have no estimates for the immediate dominators.
    fn new(cfg: &'a ControlFlowGraph<'m>) -> Self {
        Self {
            cfg,
            rpo_numbers: Dfs::new(&cfg.graph, CFGNode::Block(cfg.entry()))
                .iter(&cfg.graph)
                .zip(1..)
                .collect(),
            idoms: HashMap::new(),
        }
    }

    /// Build the dominator tree
    fn build(mut self) -> DiGraphMap<CFGNode<'m>, ()> {
        // algorithm heavily inspired by the domtree algorithm in Cranelift,
        // which itself is Keith D. Cooper's "Simple, Fast, Dominator Algorithm"
        // according to comments in Cranelift's code.

        // first compute initial (preliminary) estimates for the immediate
        // dominator of each block
        for block in Dfs::new(&self.cfg.graph, CFGNode::Block(self.cfg.entry())).iter(&self.cfg.graph) {
            self.idoms.insert(block, self.compute_idom(block));
        }

        let mut changed = true;
        while changed {
            changed = false;
            for block in Dfs::new(&self.cfg.graph, CFGNode::Block(self.cfg.entry())).iter(&self.cfg.graph) {
                let idom = self.compute_idom(block);
                let prev_idom = self.idoms.get_mut(&block).expect("All nodes in the dfs should have an initialized idom by now");
                if idom != *prev_idom {
                    *prev_idom = idom;
                    changed = true;
                }
            }
        }

        DiGraphMap::from_edges(
            self.idoms.into_iter().filter_map(|(block, idom)| Some((CFGNode::Block(idom?), block)))
        )
    }

    /// Compute the immediate dominator for `block` using the current `idom`
    /// states for the nodes.
    ///
    /// `block` must be reachable in the CFG. Returns `None` only for the entry
    /// block.
    fn compute_idom(&self, block: CFGNode<'m>) -> Option<&'m Name> {
        debug!("domtree: compute_idom for {}", block);
        if block == CFGNode::Block(self.cfg.entry()) {
            return None;
        }
        // technically speaking, these are just the reachable preds which already have an idom estimate
        let mut reachable_preds = self.cfg
            .preds_of_cfgnode(block)
            .filter(|block| self.idoms.contains_key(&CFGNode::Block(block)));

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
            match self.rpo_numbers[&CFGNode::Block(block_a)].cmp(&self.rpo_numbers[&CFGNode::Block(block_b)]) {
                Ordering::Less => {
                    block_b = self.idoms[&CFGNode::Block(block_b)].unwrap_or(self.cfg.entry());
                },
                Ordering::Greater => {
                    block_a = self.idoms[&CFGNode::Block(block_a)].unwrap_or(self.cfg.entry());
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
        let mut parents = self.graph.neighbors_directed(CFGNode::Block(block), Direction::Incoming);
        let idom = parents.next()?;
        if let Some(_) = parents.next() {
            panic!("Block {:?} should have only one immediate dominator");
        }
        match idom {
            CFGNode::Block(block) => Some(block),
            CFGNode::Return => panic!("Return node shouldn't be the immediate dominator of anything"),
        }
    }

    /// Get the immediate dominator of `CFGNode::Return`.
    ///
    /// This will be the block bbX such that:
    ///   - bbX strictly dominates `CFGNode::Return`, i.e., bbX appears on every
    ///     control-flow path through the function (but bbX =/= `CFGNode::Return`)
    ///   - Of the blocks that strictly dominate `CFGNode::Return`, bbX is the
    ///     closest to `CFGNode::Return` (farthest from entry) along paths through
    ///     the function
    pub fn idom_of_return(&self) -> &'m Name {
        let mut parents = self.graph.neighbors_directed(CFGNode::Return, Direction::Incoming);
        let idom = parents.next().expect("Return node should have an idom");
        if let Some(_) = parents.next() {
            panic!("Return node should have only one immediate dominator");
        }
        match idom {
            CFGNode::Block(block) => block,
            CFGNode::Return => panic!("Return node shouldn't be its own immediate dominator"),
        }
    }

    /// Get the children of the given basic block in the dominator tree, i.e.,
    /// get all the blocks which are immediately dominated by `block`.
    ///
    /// See notes on `idom()`.
    pub fn children<'s>(&'s self, block: &'m Name) -> impl Iterator<Item = CFGNode<'m>> + 's {
        self.graph.neighbors_directed(CFGNode::Block(block), Direction::Outgoing)
    }

    /// Get the `Name` of the entry block for the function
    pub fn entry(&self) -> &'m Name {
        self.entry_node
    }
}
