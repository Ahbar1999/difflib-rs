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

/*
#[macro_export]
macro_rules! chars_to_strs {
    ($s:expr) => {
        {
            let mut buf = vec![0 as u8; $s.len()];
        
            let mut v = Vec::new();
        
            for (i, ch) in $s.chars().enumerate() {
                ch.encode_utf8(&mut buf[i..i +1]);
            }

            for i in 0..$s.len() {
                v.push(str::from_utf8(&buf[i..i + 1]).ok().unwrap());
            }
                
            v
        }
    }
}
*/
