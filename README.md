# `llvm-ir-analysis`: Static analysis of LLVM IR

This crate provides several simple static analyses of LLVM IR.
In particular, this crate computes the following on an [`llvm-ir`] `Module`:

- [`CallGraph`](https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.CallGraph.html)
- [`ControlFlowGraph`](https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.ControlFlowGraph.html)
- [`DominatorTree`](https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.DominatorTree.html)
- [`PostDominatorTree`](https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.PostDominatorTree.html)
- [`ControlDependenceGraph`](https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.ControlDependenceGraph.html)
- [`FunctionsByType`](https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.FunctionsByType.html)

All of the above analyses are provided by an [`Analysis`] object which lazily
computes each of these structures on demand and caches the results.

## Getting started

`llvm-ir-analysis` is on [crates.io](https://crates.io/crates/llvm-ir-analysis),
so you can simply add it as a dependency in your `Cargo.toml`, selecting the
feature corresponding to the LLVM version you want:
```toml
[dependencies]
llvm-ir-analysis = { version = "0.1.0", features = ["llvm-10"] }
```
As of 0.1.0, the supported LLVM versions are `llvm-8`, `llvm-9`, and
`llvm-10`; the current Git `master` also supports `llvm-11`.
The corresponding LLVM library must be available on your system; see the
[`llvm-sys`] README for more details and instructions.

You'll also need some LLVM IR to analyze, in the form of an [`llvm-ir`] [`Module`].
This can be easily generated from an LLVM bitcode file; for more detailed
instructions, see [`llvm-ir`'s README](https://crates.io/crates/llvm-ir).

Then, you can construct an [`Analysis`] object:
```rust
let module = Module::from_bc_path(...)?;
let analysis = Analysis::new(&module);
```

You can get `Module`-wide analyses such as `analysis.call_graph()`
directly from the `Analysis` object.
You can also get `Function`-level analyses such as the control-flow
graph using `analysis.fn_analysis("my_func")`; or you can construct
a [`FunctionAnalysis`] directly with `FunctionAnalysis::new()`.

[`llvm-ir`]: https://crates.io/crates/llvm-ir
[`llvm-sys`]: https://crates.io/crates/llvm-sys
[`Module`]: https://docs.rs/llvm-ir/0.7.3/llvm_ir/module/struct.Module.html
[`Analysis`]: https://docs.rs/llvm-ir-analysis/0.1.0/llvm_ir_analysis/struct.Analysis.html
[`FunctionAnalysis`]: https://docs.rs/llvm-ir-analysis/0.2.0/llvm_ir_analysis/struct.FunctionAnalysis.html
