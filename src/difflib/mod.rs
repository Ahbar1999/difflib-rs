// single file codebase for all of difflib-go port
use std::collections::{HashMap};
use std::cmp::{max, min};

struct Match {
    a: usize, 
    b: usize, 
    size: usize,
}

impl Match {
    fn new() -> Self {
        Self {
            a: 0, 
            b: 0,
            size: 0
        }
    }
}

struct OpCode {
    tag: u8,
    i1: usize,
    i2: usize,
    j1: usize,
    j2: usize,
}

impl OpCode { 
    fn new(tag: u8, i1: usize, i2: usize, j1: usize, j2: usize) -> Self {
        Self {
            tag,
            i1,
            i2,
            j1,
            j2
        }
    }
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
    a: Option<Vec<&'life_of_a str>>,
    b: Option<Vec<&'life_of_b str>>,
    b2j: HashMap<&'life_of_b str, Vec<usize>>,          // line -> index mapping for b sequence
    is_junk: Option<Box<dyn Fn(&'_ str) -> bool>>,      // a function for checking junk lines 
    auto_junk: bool,                                        
    b_junk: HashMap<&'life_of_b str, Match>,            // depository of junks in b
    matching_blocks: Vec<Match>,
    full_b_count: HashMap<&'life_of_b str, usize>,
    b_popular: HashMap<&'life_of_b str, Match>,
    op_codes: Vec<OpCode>,
}

impl<'life_of_self, 'life_of_b, 'life_of_a> SequenceMatcher<'life_of_a, 'life_of_b> {
    // todo:
    // convert a,b to Option<&_>, it will require adding another lifetime but it should work
    // allowing us to not copy these vectors
    pub fn new(_a: Vec<&'life_of_a str>, _b: Vec<&'life_of_b str>) -> Self {
        let mut m = Self {
            a: None,
            b: None,
            b2j: HashMap::new(),
            is_junk: None,
            auto_junk: true,
            b_junk: HashMap::new(),
            matching_blocks: Vec::new(),
            full_b_count: HashMap::new(),
            b_popular: HashMap::new(),
            op_codes: Vec::new()
        };

        m.set_seqs(_a.clone(), _b.clone());

        m
    }

    fn set_seqs(&mut self, a: Vec<&'life_of_a str>, b: Vec<&'life_of_b str>) {
        self.set_seq1(a);
        self.set_seq2(b);
    } 

    fn set_seq1(&mut self, a: Vec<&'life_of_a str>) {
        if a == *self.a.as_ref().unwrap() {
            return;
        }

        self.a = Some(a);
        self.matching_blocks = Vec::new();
        self.op_codes = Vec::new(); 
    }
     
    fn set_seq2(&mut self, b: Vec<&'life_of_b str>) {
        if b == *self.b.as_ref().unwrap() {
            return;
        }

        self.b = Some(b);
        self.matching_blocks = Vec::new();
        self.op_codes = Vec::new();
        self.full_b_count = HashMap::new();
        self.chain_b();
    }
     
    pub fn new_with_junk(
        _a: Vec<&'life_of_a str>, 
        _b: Vec<&'life_of_b str>,
        is_junk: Box<dyn Fn(&'_ str) -> bool>) -> Self {
        
        let mut matcher = SequenceMatcher::new(_a, _b);
        
        matcher.is_junk.replace(is_junk);

        matcher
    }

    fn chain_b(&mut self) {
        for (i, s) in self.b.as_ref().unwrap().iter().enumerate() {
            self.b2j
                .entry(s)
                .and_modify(|v| { v.push(i); })
                .or_insert(vec![i]);
        }

        // remove junk elements if is_junk detector was provided
        if !self.is_junk.is_some() {
            // store junks separately  
            for (s, _) in self.b2j.iter() {
                // call is_junk(s)
                if self.is_junk.as_ref().unwrap()(s) {
                    self.b_junk.insert(s, Match::new());
                }
            } 
            
            // remove junks from b2j
            for (s, _) in self.b_junk.iter() {
                self.b2j.remove(s);
            }
        }

        let n = self.b.as_ref().unwrap().len();
        
        // purge popular lines
        if self.auto_junk && n >= 200 {
            let ntest = n/ 100 + 1;

            for (s, indices) in self.b2j.iter() {
                if indices.len() > ntest {
                    self.b_popular.insert(s, Match::new());
                }
            } 
            
            for (s, _) in self.b_popular.iter() {
                self.b2j.remove(s);
            }
        }
    }

    pub fn is_b_junk(&self, s: &'life_of_b str) -> bool {
        self.b_junk.contains_key(s)
    }
    
    fn find_longest_match(&self, alo: usize, ahi: usize, blo: usize, bhi: usize) -> Match {
        let mut besti: usize =0;
        let mut bestj: usize =0;
        let mut bestsize: usize =0;
        
        /*
            find the longest junk-free match
            during an iteration of the loop, j2len[j] = length of longest
            junk-free match ending with a[i - 1] and b[j]
        */
        let mut j2len = HashMap::<usize, usize>::new();
        
        for i in alo..ahi {
            // look at all instances of a[i] in b; note that because
            // b2j has no junk keys, the loop is skipped if a[i] is junk
            let mut newj2len = HashMap::new();

            for j in self.b2j[self.a.as_ref().unwrap()[i]].iter() {
                if *j < blo {
                    continue;
                }
                if *j >= bhi {
                    break; 
                }

                let k = j2len[&(j -1)] + 1;
                newj2len.insert(*j, k);
                if k > bestsize {
                    besti = i - k +1;
                    bestj = *j - k+  1;
                    bestsize = k;
                } 
            }
            j2len = newj2len;
        }

        // Extend the best by non-junk elements on each end.  In particular,
        // "popular" non-junk elements aren't in b2j, which greatly speeds
        // the inner loop above, but also means "the best" match so far
        // doesn't contain any junk *or* popular non-junk elements.
        
        while besti > alo 
            && bestj > blo 
            && !self.is_b_junk(self.b.as_ref().unwrap()[bestj - 1])
            && self.a.as_ref().unwrap()[besti - 1] == self.b.as_ref().unwrap()[bestj - 1] {
            bestsize += 1;    
        }

        // Now that we have a wholly interesting match (albeit possibly
        // empty!), we may as well suck up the matching junk on each
        // side of it too.  Can't think of a good reason not to, and it
        // saves post-processing the (possibly considerable) expense of
        // figuring out what to do with it.  In the case of an empty
        // interesting match, this is clearly the right thing to do,
        // because no other kind of match is possible in the regions.
        while besti > alo && bestj > blo && self.is_b_junk(self.b.as_ref().unwrap()[bestj - 1]) 
            && self.a.as_ref().unwrap()[besti - 1] == self.b.as_ref().unwrap()[bestj - 1] {
            besti -= 1;
            bestj -= 1;
            bestsize += 1;
        }  

        while besti + bestsize < ahi && bestj + bestsize < bhi 
            && self.is_b_junk(self.b.as_ref().unwrap()[bestj + bestsize])
            && self.a.as_ref().unwrap()[besti + bestsize] == self.b.as_ref().unwrap()[bestj + bestsize] {
            bestsize += 1; 
        } 

        return Match {
            a: besti,
            b: bestj,
            size: bestsize
        }
    }

    // Return list of triples describing matching subsequences.
    //
    // Each triple is of the form (i, j, n), and means that
    // a[i:i+n] == b[j:j+n].  The triples are monotonically increasing in
    // i and in j. It's also guaranteed that if (i, j, n) and (i', j', n') are
    // adjacent triples in the list, and the second is not the last triple in the
    // list, then i+n != i' or j+n != j'. IOW, adjacent triples never describe
    // adjacent equal blocks.
    //
    // The last triple is a dummy, (len(a), len(b), 0), and is the only
    // triple with n==0.
    fn match_blocks(&'life_of_self self, alo: usize, ahi: usize, blo: usize, bhi: usize, mut matched: &'life_of_self mut Vec<Match>) -> &'life_of_self mut Vec<Match> {
        let _match = self.find_longest_match(alo, ahi, blo, bhi);
        let i = _match.a;
        let j = _match.b;
        let k = _match.size;
        
        if _match.size > 0 {
            if alo < i && blo < j { 
                matched = self.match_blocks(alo, i, blo, j, matched);
            }
            matched.push(_match);

            if i + k < ahi && j + k < bhi {
                matched = self.match_blocks(i + k, ahi, j + k, bhi, matched);
            }
        }

        matched
    }

    fn get_matching_blocks(&mut self) -> &Vec<Match> {
        if !self.matching_blocks.is_empty() {
            return self.matching_blocks.as_ref();
        }
        
        let mut m = Vec::new();
        let matched = self.match_blocks(
            0, 
            self.a.as_ref().unwrap().len(),
            0, 
            self.b.as_ref().unwrap().len(),
            &mut m);
        
        // It's possible that we have adjacent equal blocks in the
        // matching_blocks list now.
        let mut non_adjacent = Vec::new();

        let mut i1 = 0 as usize;
        let mut j1 = i1;
        let mut k1 = j1;

        for b in matched.iter() {
            // Is this block adjacent to i1, j1, k1?
            let i2 = b.a;
            let j2 = b.b;
            let k2 = b.size;

            if i1 + k1 == i2 && j1 + k1 == j2 {
                // Yes, so collapse them -- this just increases the length of
                // the first block by the length of the second, and the first
                // block so lengthened remains the block to compare against.
                k1 += k2
            } else {
                // Not adjacent.  Remember the first block (k1==0 means it's
                // the dummy we started with), and make the second block the
                // new block to compare against.
                if k1 > 0 {
                    non_adjacent.push(Match{
                        a: i1,
                        b: j1, 
                        size: k1
                    });
                }
                i1 = i2;
                j1 = j2;
                k1 = k2;
            }
        }

        if k1 > 0 {
            non_adjacent.push(Match {
                a: i1, 
                b: j1, 
                size: k1
            });
        }
        
        non_adjacent.push(Match {
            a: self.a.as_ref().unwrap().len(),
            b: self.b.as_ref().unwrap().len(),
            size: 0
        });
        self.matching_blocks = non_adjacent; 

        self.matching_blocks.as_ref()
    }

    // Return list of 5-tuples describing how to turn a into b.
    //
    // Each tuple is of the form (tag, i1, i2, j1, j2).  The first tuple
    // has i1 == j1 == 0, and remaining tuples have i1 == the i2 from the
    // tuple preceding it, and likewise for j1 == the previous j2.
    //
    // The tags are characters, with these meanings:
    //
    // 'r' (replace):  a[i1:i2] should be replaced by b[j1:j2]
    //
    // 'd' (delete):   a[i1:i2] should be deleted, j1==j2 in this case.
    //
    // 'i' (insert):   b[j1:j2] should be inserted at a[i1:i1], i1==i2 in this case.
    //
    // 'e' (equal):    a[i1:i2] == b[j1:j2]
    fn get_op_codes(&mut self) -> &Vec<OpCode> {
        if !self.op_codes.is_empty() {
            return &self.op_codes;
        }
        let i =0 as usize;
        let j =0 as usize;
        
        let mut op_codes_v = Vec::new();
        for m in self.get_matching_blocks().iter() {
            //  invariant:  we've pumped out correct diffs to change
            //  a[:i] into b[:j], and the next matching block is
            //  a[ai:ai+size] == b[bj:bj+size]. So we need to pump
            //  out a diff to change a[i:ai] into b[j:bj], pump out
            //  the matching block, and move (i,j) beyond the match
            
            let mut tag = 0 as u8;

            if i < m.a && j < m.b {
                tag = 'r' as u8;
            } else if i < m.a {
                tag = 'd' as u8;
            } else if j < m.b {
                tag = 'i' as u8;
            } 
            
            if tag > 0 {
                op_codes_v.push(OpCode {
                    tag: tag,
                    i1: i,
                    i2: m.a,
                    j1: j,
                    j2: m.b
                }); 
            }
        }
        
        self.op_codes = op_codes_v;  
        &self.op_codes 
    }
    
    // Isolate change clusters by eliminating ranges with no changes.
    //
    // Return a generator of groups with up to n lines of context.
    // Each group is in the same format as returned by GetOpCodes().
    fn get_grouped_op_codes(&mut self, mut n: usize) -> Vec<Vec<OpCode>> {
        if n < 0 {
            n = 3;
        }
        
        let codes = &mut self.op_codes;
        if codes.len() == 0 {
            codes.push(OpCode::new(b'e',0,1,0,1));
        } 

        // Fixup leading and trailing groups if they show no changes.
        if codes[0].tag == b'e' {
            let c = &codes[0];
            let i1 = c.i1;
            let i2 = c.i2;
            let j1 = c.j1;
            let j2 = c.j2;

            codes[0] = OpCode{
                tag: c.tag,
                i1: max(i1, i2 - n),
                i2: i2,
                j1: max(j1, j2 -n),
                j2: j2,
            }
        }

        if codes.last().unwrap().tag == b'e' {
            let c = codes.last().unwrap();

            let i1 = c.i1;
            let i2 = c.i2;
            let j1 = c.j1;
            let j2 = c.j2;

            *codes.last_mut().unwrap() = OpCode::new(
                c.tag, 
                i1,
                min(i2, i1 + n), 
                j1,
                min(j2, j1 + n));
        }
        
        let nn = n + n;
        let mut groups = Vec::<_>::new();
        let mut group = Vec::new();

        for c in codes.iter() {
            let mut i1 = c.i1;
            let i2 = c.i2;
            let mut j1 = c.j1;
            let j2 = c.j2;

            // End the current group and start a new one whenever
            // there is a large range with no changes.
            if c.tag == b'e' && i2 - i1 > nn {
                group.push(OpCode::new(c.tag, i1, min(i2, i1 + n), j1, min(j2, j1 +n)));
                groups.push(group);
                group = Vec::new();
                i1 = max(i1, i2 - n);
                j1 = max(j1, j2 - n);
            }
            group.push(OpCode::new(c.tag, i1, i2, j1, j2)); 
        }

        if group.len() > 0 && !(group.len() == 1 && group[0].tag == b'e') {
            groups.push(group);
        }
        
        groups 
    } 
    
    // Return a measure of the sequences' similarity (float in [0,1]).
    //
    // Where T is the total number of elements in both sequences, and
    // M is the number of matches, this is 2.0*M / T.
    // Note that this is 1 if the sequences are identical, and 0 if
    // they have nothing in common.
    //
    // .ratio() is expensive to compute if you haven't already computed
    // .get_matching_blocks() or .get_op_codes(), in which case you may
    // want to try .quick_ratio() or .real_quick_ratio() first to get an
    // upper bound.
    fn ratio(&mut self) -> f64 {
        let mut matches =0;

        for m in self.get_matching_blocks() {
            matches += m.size;
        }

        calculate_ratio(matches, self.a.as_ref().unwrap().len() + self.b.as_ref().unwrap().len())
    } 
    
    // Return an upper bound on ratio() relatively quickly.
    //
    // This isn't defined beyond that it is an upper bound on .Ratio(), and
    // is faster to compute.
    fn quick_ratio(&mut self) -> f64 {
        // viewing a and b as multisets, set matches to the cardinality
        // of their intersection; this counts the number of matches
        // without regard to order, so is clearly an upper bound
        
        if self.full_b_count.is_empty() {
            for s in self.b.as_ref().unwrap() {
                self.full_b_count
                    .entry(s)
                    .and_modify(|value| { *value += 1; })
                    .or_insert(0); 
            } 
        }

        // avail[x] is the number of times x appears in 'b' less the
        // number of times we've seen it in 'a' so far ... kinda
        let mut avail = HashMap::new();
        let mut matches =0;
        
        for s in self.a.as_ref().unwrap() {
            let n = *avail
                .entry(s)
                .or_insert(self.full_b_count[s]);
            
            // update the entry
            avail.insert(s,n -1);

            if n > 0 {
                matches += 1;
            }
        }

        return calculate_ratio(
            matches, 
            self.a.as_ref().unwrap().len() + self.b.as_ref().unwrap().len());
    }

    // Return an upper bound on ratio() very quickly.
    //
    // This isn't defined beyond that it is an upper bound on .ratio(), and
    // is faster to compute than either .ratio() or .quick_ratio().
    fn real_quick_ratio(&self) -> f64 {
        let la = self.a.as_ref().unwrap().len();
        let lb = self.b.as_ref().unwrap().len();
    
        return calculate_ratio(min(la, lb), la + lb); 
    }

    // Convert range to the "ed" format
    fn format_range_unified(start: usize, stop: usize) -> String {
        // Per the diff spec at http://www.unix.org/single_unix_specification/
        let mut beginning = start + 1;    // lines start numbering with one
        let length = stop - start;

        if length == 1 {
            return format!("{:?}", beginning);
        }

        if length == 0 {
            beginning -= 1; // empty ranges begin at line just before the range
        }
        
        return format!("{:?},{:?}", beginning, length);
    }

    // Unified diff parameters
}

pub struct UnifiedDiff<'life_of_a, 'life_of_b, 'life_of_this> {
    pub a:          Option<&'life_of_this Vec<&'life_of_a str>>,     // first sequence line 
    pub from_file:  String,
    pub from_date:  String,
    pub b:          Option<&'life_of_this Vec<&'life_of_b str>>,
    pub to_file:    String,
    pub to_date:    String,
    pub eol:        String,
    pub context:    usize
}

impl<'life_of_a, 'life_of_b, 'life_of_this> UnifiedDiff<'life_of_a, 'life_of_b, 'life_of_this> {
    fn new() -> Self {
        Self {
            a: None,
            from_file: String::new(),
            from_date: String::new(),
            b: None,
            to_file: String::new(),
            to_date: String::new(),
            eol: String::new(),
            context: 0
        }
    }
}

// Convert range to the "ed" format
fn format_range_unified(start: usize, stop: usize) -> String {
    // Per the diff spec at http://www.unix.org/single_unix_specification/
	let mut beginning = start + 1; // lines start numbering with one
	let length = stop - start;

	if length == 1 {
		return format!("{}", beginning);
	}

	if length == 0 {
		beginning -= 1 // empty ranges begin at line just before the range
	}

	format!("{},{}", beginning, length)
}

pub fn write_unified_diff(mut writer: impl std::io::Write, diff: &mut UnifiedDiff) -> std::io::Result<()> {
    
    if diff.eol.len() == 0 {
        diff.eol = "\n".to_string();
    }

    let mut started = false;
    let mut m = SequenceMatcher::new(
        diff.a.as_ref().unwrap().to_vec(), 
        diff.a.as_ref().unwrap().to_vec()); 
    
    for g in m.get_grouped_op_codes(diff.context) { 
        if !started {
            started = true;
            let mut from_date = String::new();
            if diff.from_date.len() > 0 { 
                from_date = "\t".to_owned() + &diff.from_date;
            } 

            let mut to_date = String::new();
            if diff.to_date.len() > 0 {
                to_date = "\t".to_owned() + &diff.to_date;
            }

            if diff.from_file != "" || diff.to_file != "" {
                writer.write(format!("--- {}{}{}", diff.from_file, from_date, diff.eol).as_bytes())?;

                writer.write(format!("+++ {}{}{}", diff.to_file, to_date, diff.eol).as_bytes())?;
            }
        }

        let first = &g[0];
        let last = g.last().unwrap();
        
        let range1 = format_range_unified(first.i1, last.i2);
        let range2 = format_range_unified(first.j2, last.j2);
        
        writer.write(format!("@@ -{} +{} @@{}", range1, range2, diff.eol).as_bytes())?;
        
        for c in g.iter() {
            let i1= c.i1;
            let i2= c.i2;
            let j1 = c.j1;
            let j2 = c.j2;
            /*
             * match causing more duplicated code in this case therefore sticking with if
             * statements
            match c.tag {
                b'e' => { 
                    for line in &diff.a.as_ref().unwrap()[i1..i2] {
                        writer.write(format!(" {:}", line).as_bytes())?;
                    }
                },
                b'r' => { 
                    for line in &diff.a.as_ref().unwrap()[i1..i2] {
                        writer.write(format!("-{:}", line).as_bytes())?;
                    }
                    
                    for line in &diff.b.as_ref().unwrap()[j1..j2] {
                        writer.write(format!("+{:}", line).as_bytes())?;
                    }
                },
                b'd' => { 
                    for line in &diff.a.as_ref().unwrap()[i1..i2] {
                        writer.write(format!("-{:}", line).as_bytes())?;
                    }
                },
                b'i' => { 
                    for line in &diff.b.as_ref().unwrap()[j1..j2] {
                        writer.write(format!("-{:}", line).as_bytes())?;
                    }
                },
            }
            */

            if c.tag == b'e' {
                for line in &diff.b.as_ref().unwrap()[j1..j2] {
                    writer.write(format!(" {}", line).as_bytes())?;
                }
                continue;
            }

            if c.tag == b'r' || c.tag == b'd' {
                for line in &diff.a.as_ref().unwrap()[i1..i2] {
                    writer.write(format!("-{}", line).as_bytes())?;
                }
            }

            if c.tag == b'r' || c.tag == b'i' {
                for line in &diff.b.as_ref().unwrap()[j1..j2] {
                    writer.write(format!("+{}", line).as_bytes())?;
                }
            }
        } 
    }

    Ok(())
} 

pub fn get_unified_diff_string(diff: &mut UnifiedDiff) -> Result<String, std::io::Error> {
    let mut buf = std::io::BufWriter::new(Vec::<u8>::new());

    write_unified_diff(&mut buf, diff)?;
        
    Ok(String::from_utf8( buf.into_inner().ok().unwrap()).expect("Found invalid utf-8 string"))
}

fn format_range_context(start: usize, stop: usize) -> String {
    unimplemented!()
}
