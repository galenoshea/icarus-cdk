use icarus_derive::*;

struct TestStruct;

impl TestStruct {
    #[icarus_tool("Tool with self parameter")]
    fn bad_tool(&self) -> Result<String, String> {
        Ok("Should fail".to_string())
    }
}

fn main() {}