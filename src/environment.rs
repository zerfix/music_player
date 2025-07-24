use std::env::var;

#[derive(Debug)]
pub struct Environment {
    pub truecolor: bool
}

impl Environment {
    pub fn init() -> Environment {
        Environment {
            truecolor: match var("COLORTERM") {
                Ok(value) => value == "truecolor",
                Err(_)    => false,
            }
        }
    }
}
