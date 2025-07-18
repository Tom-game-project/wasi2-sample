#[allow(warnings)]
mod bindings;

//use bindings::Guest;
use crate::bindings::exports::component::tom::user_funcs::Guest;

struct Component;


impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World! This is Tom".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
