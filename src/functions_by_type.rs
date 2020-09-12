use llvm_ir::{Module, TypeRef};
use std::collections::{HashMap, HashSet};

/// Allows you to iterate over all the functions in the `Module` with a specified
/// type
pub struct FunctionsByType<'m> {
    map: HashMap<TypeRef, HashSet<&'m str>>,
}

impl<'m> FunctionsByType<'m> {
    pub(crate) fn new(module: &'m Module) -> Self {
        let mut map: HashMap<TypeRef, HashSet<&'m str>> = HashMap::new();
        for func in &module.functions {
            map.entry(module.type_of(func)).or_default().insert(&func.name);
        }
        Self {
            map,
        }
    }

    /// Iterate over all of the functions in the `Module` with the specified type
    pub fn functions_with_type<'s>(&'s self, ty: &TypeRef) -> impl Iterator<Item = &'m str> + 's {
        self.map.get(ty).into_iter().map(|hs| hs.iter().copied()).flatten()
    }
}
