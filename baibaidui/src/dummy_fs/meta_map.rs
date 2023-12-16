use std::collections::HashMap;

pub struct MetaMap {
    file_name_2_block_cnt: HashMap<String, u32>,
}

impl MetaMap {
    pub fn new(file_name_2_block_cnt: HashMap<String, u32>) -> Self {
        Self {
            file_name_2_block_cnt,
        }
    }
    pub fn file_block_cnt(&self, filename: &str) -> u32 {
        *self.file_name_2_block_cnt.get(filename).unwrap()
    }
}
