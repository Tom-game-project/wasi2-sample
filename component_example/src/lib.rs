#[allow(warnings)]
mod bindings;

//use bindings::component::tom::host_trait::say_hello;

use bindings::host::hello_world::host_trait::say_hello;

//use bindings::Guest;
use crate::bindings::exports::component::tom::user_funcs::Guest;


struct Component;


impl Guest for Component {
    /// Say hello!
    fn hello_world(name:String) -> String {
        //host_funcs::name();
        format!("Hello, World! \nsay_hello function return is \"{}\"", 
            say_hello(name.as_str())
            .as_str()
        )
    }
}

bindings::export!(Component with_types_in bindings);
