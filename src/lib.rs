pub mod difflib;
pub mod utils;


#[cfg(test)]
mod tests {
    use crate::difflib::*;
    
    fn chars_to_strs<'life_of_buf>(s: &str, buf: &'life_of_buf mut [u8]) -> Vec<&'life_of_buf str> {
        let mut v = Vec::new();
        
        for (i, ch) in s.chars().enumerate() {
            ch.encode_utf8(&mut buf[i..i +1]);
        }

        for i in 0..s.len() {
            v.push(str::from_utf8(&buf[i..i + 1] as &'life_of_buf [u8]).ok().unwrap());
        }
            
        v
    }
    
    #[test]
    fn test_get_opt_codes() {
        const A: &str = "qabxcd";
        let mut buf_a  = [0 as u8; A.len()]; 
        
        const B: &str = "abycdf";
        let mut buf_b  = [0 as u8; B.len()];   // these buffers should be heap allocated
                                                        // but for this example size we are okay 
        let mut s = SequenceMatcher::new(
             chars_to_strs(A, &mut buf_a),
             chars_to_strs(B, &mut buf_b)
        ); 
        let mut result_buf = Vec::new(); 
        for op in s.get_op_codes() {
            result_buf.push(format!("{} a[{}:{}], ({}) b[{}:{}] ({})\n", 
                op.tag as char, op.i1, op.i2, &A[op.i1..op.i2], op.j1, op.j2, &B[op.j1..op.j2])); 
        }
        
        let result: String = result_buf.into_iter().collect();

        let expected  = "d a[0:1], (q) b[0:0] ()\ne a[1:3], (ab) b[0:2] (ab)\nr a[3:4], (x) b[2:3] (y)\ne a[4:6], (cd) b[3:5] (cd)\ni a[6:6], () b[5:6] (f)\n";

        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_sequence_matcher_ratio() {
        const A: &str = "abcd";
        const B: &str = "bcde";
        let mut buf_a = [0 as u8; A.len()];
        let mut buf_b = [0 as u8; B.len()];

        let mut sm = SequenceMatcher::new(
            chars_to_strs(A, &mut buf_a),
            chars_to_strs(B, &mut buf_b));
        
        assert_eq!(sm.ratio(), 0.75);
        assert_eq!(sm.quick_ratio(), 0.75);
        assert_eq!(sm.real_quick_ratio(), 1.0);
    }

    #[test]
    fn test_grouped_codes() {
        let mut x = Vec::new();
         
        (0..39).into_iter().for_each(|i| x.push(format!("{}", i)) );

        let a= x.iter().map(|s| s.as_str()).collect();

        let mut b: Vec<&str> = Vec::new();
        x[..8].iter().for_each(|s| { b.push(s); } );
        b.push(" i");
        x[8..19].iter().for_each(|s| { b.push(s); });
        b.push(" x");
        x[20..22].iter().for_each(|s| { b.push(s); });
        x[27..34].iter().for_each(|s| { b.push(s); });
        b.push(" y");
        x[35..].iter().for_each(|s| { b.push(s); });

        let mut sm = SequenceMatcher::new(a, b); 
        
        let mut result = String::new();
        for g in sm.get_grouped_op_codes(usize::MAX) {
            result.push_str(&format!("group\n"));
            for op in g {
                result.push_str(&format!(" {}, {}, {}, {}, {}\n", 
                    op.tag as char, op.i1, op.i2, op.j1, op.j2));
            }
        }

        let expected = "group\n e, 5, 8, 5, 8\n i, 8, 8, 8, 9\n e, 8, 11, 9, 12\ngroup\n e, 16, 19, 17, 20\n r, 19, 20, 20, 21\n e, 20, 22, 21, 23\n d, 22, 27, 23, 23\n e, 27, 30, 23, 26\ngroup\n e, 31, 34, 27, 30\n r, 34, 35, 30, 31\n e, 35, 38, 31, 34\n";
        assert_eq!(result.as_str(), expected);
    }

/*
func ExampleGetUnifiedDiffCode() {
	a := `one
two
three
four
fmt.Printf("%s,%T",a,b)`
	b := `zero
one
three
four`
	diff := UnifiedDiff{
		A:        SplitLines(a),
		B:        SplitLines(b),
		FromFile: "Original",
		FromDate: "2005-01-26 23:30:50",
		ToFile:   "Current",
		ToDate:   "2010-04-02 10:20:52",
		Context:  3,
	}
	result, _ := GetUnifiedDiffString(diff)
	fmt.Println(strings.Replace(result, "\t", " ", -1))
	// Output:
	// --- Original 2005-01-26 23:30:50
	// +++ Current 2010-04-02 10:20:52
	// @@ -1,5 +1,4 @@
	// +zero
	//  one
	// -two
	//  three
	//  four
	// -fmt.Printf("%s,%T",a,b)
}

func ExampleGetContextDiffCode() {
	a := `one
two
three
four
fmt.Printf("%s,%T",a,b)`
	b := `zero
one
tree
four`
	diff := ContextDiff{
		A:        SplitLines(a),
		B:        SplitLines(b),
		FromFile: "Original",
		ToFile:   "Current",
		Context:  3,
		Eol:      "\n",
	}
	result, _ := GetContextDiffString(diff)
	fmt.Print(strings.Replace(result, "\t", " ", -1))
	// Output:
	// *** Original
	// --- Current
	// ***************
	// *** 1,5 ****
	//   one
	// ! two
	// ! three
	//   four
	// - fmt.Printf("%s,%T",a,b)
	// --- 1,4 ----
	// + zero
	//   one
	// ! tree
	//   four
}

func ExampleGetContextDiffString() {
	a := `one
two
three
four`
	b := `zero
one
tree
four`
	diff := ContextDiff{
		A:        SplitLines(a),
		B:        SplitLines(b),
		FromFile: "Original",
		ToFile:   "Current",
		Context:  3,
		Eol:      "\n",
	}
	result, _ := GetContextDiffString(diff)
	fmt.Printf(strings.Replace(result, "\t", " ", -1))
	// Output:
	// *** Original
	// --- Current
	// ***************
	// *** 1,4 ****
	//   one
	// ! two
	// ! three
	//   four
	// --- 1,4 ----
	// + zero
	//   one
	// ! tree
	//   four
}

func rep(s string, count int) string {
	return strings.Repeat(s, count)
}

func TestWithAsciiOneInsert(t *testing.T) {
	sm := NewMatcher(splitChars(rep("b", 100)),
		splitChars("a"+rep("b", 100)))
	assertAlmostEqual(t, sm.Ratio(), 0.995, 3)
	assertEqual(t, sm.GetOpCodes(),
		[]OpCode{{'i', 0, 0, 0, 1}, {'e', 0, 100, 1, 101}})
	assertEqual(t, len(sm.bPopular), 0)

	sm = NewMatcher(splitChars(rep("b", 100)),
		splitChars(rep("b", 50)+"a"+rep("b", 50)))
	assertAlmostEqual(t, sm.Ratio(), 0.995, 3)
	assertEqual(t, sm.GetOpCodes(),
		[]OpCode{{'e', 0, 50, 0, 50}, {'i', 50, 50, 50, 51}, {'e', 50, 100, 51, 101}})
	assertEqual(t, len(sm.bPopular), 0)
}

func TestWithAsciiOnDelete(t *testing.T) {
	sm := NewMatcher(splitChars(rep("a", 40)+"c"+rep("b", 40)),
		splitChars(rep("a", 40)+rep("b", 40)))
	assertAlmostEqual(t, sm.Ratio(), 0.994, 3)
	assertEqual(t, sm.GetOpCodes(),
		[]OpCode{{'e', 0, 40, 0, 40}, {'d', 40, 41, 40, 40}, {'e', 41, 81, 40, 80}})
}

func TestWithAsciiBJunk(t *testing.T) {
	isJunk := func(s string) bool {
		return s == " "
	}
	sm := NewMatcherWithJunk(splitChars(rep("a", 40)+rep("b", 40)),
		splitChars(rep("a", 44)+rep("b", 40)), true, isJunk)
	assertEqual(t, sm.bJunk, map[string]struct{}{})

	sm = NewMatcherWithJunk(splitChars(rep("a", 40)+rep("b", 40)),
		splitChars(rep("a", 44)+rep("b", 40)+rep(" ", 20)), false, isJunk)
	assertEqual(t, sm.bJunk, map[string]struct{}{" ": struct{}{}})

	isJunk = func(s string) bool {
		return s == " " || s == "b"
	}
	sm = NewMatcherWithJunk(splitChars(rep("a", 40)+rep("b", 40)),
		splitChars(rep("a", 44)+rep("b", 40)+rep(" ", 20)), false, isJunk)
	assertEqual(t, sm.bJunk, map[string]struct{}{" ": struct{}{}, "b": struct{}{}})
}

func TestSFBugsRatioForNullSeqn(t *testing.T) {
	sm := NewMatcher(nil, nil)
	assertEqual(t, sm.Ratio(), 1.0)
	assertEqual(t, sm.QuickRatio(), 1.0)
	assertEqual(t, sm.RealQuickRatio(), 1.0)
}

func TestSFBugsComparingEmptyLists(t *testing.T) {
	groups := NewMatcher(nil, nil).GetGroupedOpCodes(-1)
	assertEqual(t, len(groups), 0)
	diff := UnifiedDiff{
		FromFile: "Original",
		ToFile:   "Current",
		Context:  3,
	}
	result, err := GetUnifiedDiffString(diff)
	assertEqual(t, err, nil)
	assertEqual(t, result, "")
}

func TestOutputFormatRangeFormatUnified(t *testing.T) {
	// Per the diff spec at http://www.unix.org/single_unix_specification/
	//
	// Each <range> field shall be of the form:
	//   %1d", <beginning line number>  if the range contains exactly one line,
	// and:
	//  "%1d,%1d", <beginning line number>, <number of lines> otherwise.
	// If a range is empty, its beginning line number shall be the number of
	// the line just before the range, or 0 if the empty range starts the file.
	fm := formatRangeUnified
	assertEqual(t, fm(3, 3), "3,0")
	assertEqual(t, fm(3, 4), "4")
	assertEqual(t, fm(3, 5), "4,2")
	assertEqual(t, fm(3, 6), "4,3")
	assertEqual(t, fm(0, 0), "0,0")
}

func TestOutputFormatRangeFormatContext(t *testing.T) {
	// Per the diff spec at http://www.unix.org/single_unix_specification/
	//
	// The range of lines in file1 shall be written in the following format
	// if the range contains two or more lines:
	//     "*** %d,%d ****\n", <beginning line number>, <ending line number>
	// and the following format otherwise:
	//     "*** %d ****\n", <ending line number>
	// The ending line number of an empty range shall be the number of the preceding line,
	// or 0 if the range is at the start of the file.
	//
	// Next, the range of lines in file2 shall be written in the following format
	// if the range contains two or more lines:
	//     "--- %d,%d ----\n", <beginning line number>, <ending line number>
	// and the following format otherwise:
	//     "--- %d ----\n", <ending line number>
	fm := formatRangeContext
	assertEqual(t, fm(3, 3), "3")
	assertEqual(t, fm(3, 4), "4")
	assertEqual(t, fm(3, 5), "4,5")
	assertEqual(t, fm(3, 6), "4,6")
	assertEqual(t, fm(0, 0), "0")
}

func TestOutputFormatTabDelimiter(t *testing.T) {
	diff := UnifiedDiff{
		A:        splitChars("one"),
		B:        splitChars("two"),
		FromFile: "Original",
		FromDate: "2005-01-26 23:30:50",
		ToFile:   "Current",
		ToDate:   "2010-04-12 10:20:52",
		Eol:      "\n",
	}
	ud, err := GetUnifiedDiffString(diff)
	assertEqual(t, err, nil)
	assertEqual(t, SplitLines(ud)[:2], []string{
		"--- Original\t2005-01-26 23:30:50\n",
		"+++ Current\t2010-04-12 10:20:52\n",
	})
	cd, err := GetContextDiffString(ContextDiff(diff))
	assertEqual(t, err, nil)
	assertEqual(t, SplitLines(cd)[:2], []string{
		"*** Original\t2005-01-26 23:30:50\n",
		"--- Current\t2010-04-12 10:20:52\n",
	})
}

func TestOutputFormatNoTrailingTabOnEmptyFiledate(t *testing.T) {
	diff := UnifiedDiff{
		A:        splitChars("one"),
		B:        splitChars("two"),
		FromFile: "Original",
		ToFile:   "Current",
		Eol:      "\n",
	}
	ud, err := GetUnifiedDiffString(diff)
	assertEqual(t, err, nil)
	assertEqual(t, SplitLines(ud)[:2], []string{"--- Original\n", "+++ Current\n"})

	cd, err := GetContextDiffString(ContextDiff(diff))
	assertEqual(t, err, nil)
	assertEqual(t, SplitLines(cd)[:2], []string{"*** Original\n", "--- Current\n"})
}

func TestOmitFilenames(t *testing.T) {
	diff := UnifiedDiff{
		A:   SplitLines("o\nn\ne\n"),
		B:   SplitLines("t\nw\no\n"),
		Eol: "\n",
	}
	ud, err := GetUnifiedDiffString(diff)
	assertEqual(t, err, nil)
	assertEqual(t, SplitLines(ud), []string{
		"@@ -0,0 +1,2 @@\n",
		"+t\n",
		"+w\n",
		"@@ -2,2 +3,0 @@\n",
		"-n\n",
		"-e\n",
		"\n",
	})

	cd, err := GetContextDiffString(ContextDiff(diff))
	assertEqual(t, err, nil)
	assertEqual(t, SplitLines(cd), []string{
		"***************\n",
		"*** 0 ****\n",
		"--- 1,2 ----\n",
		"+ t\n",
		"+ w\n",
		"***************\n",
		"*** 2,3 ****\n",
		"- n\n",
		"- e\n",
		"--- 3 ----\n",
		"\n",
	})
}

func TestSplitLines(t *testing.T) {
	allTests := []struct {
		input string
		want  []string
	}{
		{"foo", []string{"foo\n"}},
		{"foo\nbar", []string{"foo\n", "bar\n"}},
		{"foo\nbar\n", []string{"foo\n", "bar\n", "\n"}},
	}
	for _, test := range allTests {
		assertEqual(t, SplitLines(test.input), test.want)
	}
}

func benchmarkSplitLines(b *testing.B, count int) {
	str := strings.Repeat("foo\n", count)

	b.ResetTimer()

	n := 0
	for i := 0; i < b.N; i++ {
		n += len(SplitLines(str))
	}
}

func BenchmarkSplitLines100(b *testing.B) {
	benchmarkSplitLines(b, 100)
}

func BenchmarkSplitLines10000(b *testing.B) {
	benchmarkSplitLines(b, 10000)
}
* 
*/

}
