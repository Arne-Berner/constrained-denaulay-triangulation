
// TODO find better name
#[derive(PartialEq)]
pub enum FoundOrAdded {
    Found(usize),
    Added(usize),
}

impl FoundOrAdded{
    pub fn value(self)->usize{
        match self {
            FoundOrAdded::Found(idx) => idx,
            FoundOrAdded::Added(idx) => idx,
        }
    }
}