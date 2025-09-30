use icarus_macros::tool;

struct MyStruct;

impl MyStruct {
    /// This should fail - methods with self are not supported
    #[tool]
    fn method_tool(&self, x: i32) -> i32 {
        x
    }
}

fn main() {}