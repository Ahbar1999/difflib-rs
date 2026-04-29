// single file codebase for all of difflib-go port
use std::collections::{HashMap};

struct Match {
    a: usize, 
    b: usize, 
    size: usize,
}

struct OpCode {
    i1: usize,
    i2: usize,
    j1: usize,
    j2: usize,
    
    tag: u8,
}

fn calculate_ratio(matches: usize, length: usize) -> f64 {
    if length > 0 {
        return 2.0 * (matches as f64) / (length as f64); 
    } 

   1.0 
}

fn split_lines(s: &str) -> Vec<&str> {
    let mut lines = Vec::new();

    for line in s.split_inclusive('\n') {
        lines.push(line);
    }
    
    lines
}

struct SequenceMatcher<'life_of_a, 'life_of_b> {
    a: Vec<&'life_of_a str>,
    b: Vec<&'life_of_b str>,
    b2j: HashMap<&'life_of_b str, Vec<usize>>,
    is_junk: Box<dyn Fn(&'_ str) -> bool>,
    auto_junk: bool,
    b_junk: HashMap<&'life_of_b str, Match>,
    matching_blocks: Vec<Match>,
    full_b_count: HashMap<&'life_of_b str, usize>,
    b_popular: HashMap<&'life_of_b str, Match>,
    op_codes: Vec<OpCode>,
}

impl<'life_of_a, 'life_of_b> SequenceMatcher<'life_of_a, 'life_of_b> {
    fn chain_b(&mut self) {
        


    }
} 
