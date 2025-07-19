#[allow(warnings)]
mod bindings;


//use bindings::component::tom::host_funcs;

//use bindings::Guest;
use crate::bindings::exports::component::tom::user_funcs::Guest;


struct Component;


impl Guest for Component {
    /// Say hello!
    fn hello_world(name:String) -> String {
        //host_funcs::name();
        format!("Hello, World! This is {}", name)
    }
}

bindings::export!(Component with_types_in bindings);
