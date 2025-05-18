//! data module for Lurhook

pub fn init() {
    println!("Initialized crate: data");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_runs() {
        init();
    }
}
