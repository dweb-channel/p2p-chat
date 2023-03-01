use std::error::Error;


struct PChat {
    pid:String,
    ip:String,
}

impl PChat {
    fn create() {

    }
}


fn main() -> Result<(),Box<dyn Error>> {
    println!("chat init");
    Ok(())
}

