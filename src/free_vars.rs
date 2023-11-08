use std::collections::HashSet;

pub trait FreeVars {
    fn free_vars(&self) -> HashSet<u32>;
}

impl<T> FreeVars for Box<[T]>
where
    T: FreeVars,
{
    fn free_vars(&self) -> HashSet<u32> {
        let mut free_vars = HashSet::new();
        for elem in self.iter() {
            free_vars.extend(elem.free_vars().into_iter());
        }
        free_vars
    }
}
