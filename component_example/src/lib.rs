#[allow(warnings)]
mod bindings;

use bindings::host::hello_world::*;

//use bindings::Guest;
use crate::bindings::exports::component::tom::user_funcs::Guest;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world(name:String) -> String {
        //host_funcs::name();
        host_trait::set_name("mot");
        format!("Hello, World! \nsay_hello function return is \"{}\"", 
            host_trait::get_name()
        )
    }
}

fn main()
{
    return ;
}

bindings::export!(Component with_types_in bindings);
