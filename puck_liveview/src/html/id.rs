#[derive(Debug, Default)]
pub struct IdGen {
    head: usize,
}

impl IdGen {
    pub fn new() -> IdGen {
        Default::default()
    }

    pub(crate) fn new_id(&mut self) -> usize {
        let ret = self.head;
        self.head += 1;
        ret
    }
}
