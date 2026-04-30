#[macro_export]
macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut m = std::collections::HashMap::new();
            // for each pair 'key', 'value' matched before, expand the following statement 
            $( m.insert($key, $value); )*
            
            m
        }
    };
}
