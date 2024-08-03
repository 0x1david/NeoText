#[macro_export]
macro_rules! repeat {
    ($statement:expr; $count:expr) => {
        {
            let count = match $count {
                Some(n) => n,
                None => 1,
            };
            for _ in 0..count {
                $statement
            }
        }
    };
}

