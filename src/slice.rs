use std;
use std::sync::Arc;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

use iter::{Bytes, Chars, Chunks, Lines};
use rope::Rope;
use str_utils::char_idx_to_byte_idx;
use tree::{Count, Node};

/// An immutable view into part of a `Rope`.
#[derive(Copy, Clone)]
pub struct RopeSlice<'a> {
    node: &'a Arc<Node>,
    start_byte: Count,
    end_byte: Count,
    start_char: Count,
    end_char: Count,
    start_line_break: Count,
    end_line_break: Count,
}

impl<'a> RopeSlice<'a> {
    pub(crate) fn new_with_range(node: &'a Arc<Node>, start: usize, end: usize) -> Self {
        assert!(start <= end);
        assert!(end <= node.text_info().chars as usize);

        // Find the deepest node that still contains the full range given.
        let mut n_start = start;
        let mut n_end = end;
        let mut node = node;
        'outer: loop {
            match *(node as &Node) {
                Node::Leaf(_) => break,

                Node::Internal(ref children) => {
                    let mut start_char = 0;
                    for (i, inf) in children.info().iter().enumerate() {
                        if n_start >= start_char && n_end < (start_char + inf.chars as usize) {
                            n_start -= start_char;
                            n_end -= start_char;
                            node = &children.nodes()[i];
                            continue 'outer;
                        }
                        start_char += inf.chars as usize;
                    }
                    break;
                }
            }
        }

        // Create the slice
        RopeSlice {
            node: node,
            start_byte: node.char_to_byte(n_start) as Count,
            end_byte: node.char_to_byte(n_end) as Count,
            start_char: n_start as Count,
            end_char: n_end as Count,
            start_line_break: node.char_to_line(n_start) as Count,
            end_line_break: node.char_to_line(n_end) as Count,
        }
    }

    //-----------------------------------------------------------------------
    // Informational methods

    /// Total number of bytes in the `RopeSlice`.
    pub fn len_bytes(&self) -> usize {
        (self.end_byte - self.start_byte) as usize
    }

    /// Total number of chars in the `RopeSlice`.
    pub fn len_chars(&self) -> usize {
        (self.end_char - self.start_char) as usize
    }

    /// Total number of lines in the `RopeSlice`.
    pub fn len_lines(&self) -> usize {
        (self.end_line_break - self.start_line_break) as usize + 1
    }

    //-----------------------------------------------------------------------
    // Index conversion methods

    /// Returns the char index of the given byte.
    ///
    /// # Panics
    ///
    /// Panics if `byte_idx` is out of bounds (i.e. `byte_idx > len_bytes()`).
    #[allow(dead_code)]
    pub fn byte_to_char(&self, byte_idx: usize) -> usize {
        // Bounds check
        assert!(
            byte_idx <= self.len_bytes(),
            "Attempt to index past end of slice: byte index {}, slice byte length {}",
            byte_idx,
            self.len_bytes()
        );

        self.node.byte_to_char(self.start_byte as usize + byte_idx) - (self.start_char as usize)
    }

    /// Returns the byte index of the given char.
    ///
    /// # Panics
    ///
    /// Panics if `char_idx` is out of bounds (i.e. `char_idx > len_chars()`).
    #[allow(dead_code)]
    pub fn char_to_byte(&self, char_idx: usize) -> usize {
        // Bounds check
        assert!(
            char_idx <= self.len_chars(),
            "Attempt to index past end of slice: char index {}, slice char length {}",
            char_idx,
            self.len_chars()
        );

        self.node.char_to_byte(self.start_char as usize + char_idx) - (self.start_byte as usize)
    }

    /// Returns the line index of the given char.
    ///
    /// # Panics
    ///
    /// Panics if `char_idx` is out of bounds (i.e. `char_idx > len_chars()`).
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        // Bounds check
        assert!(
            char_idx <= self.len_chars(),
            "Attempt to index past end of slice: char index {}, slice char length {}",
            char_idx,
            self.len_chars()
        );

        if char_idx == self.len_chars() {
            self.len_lines()
        } else {
            self.node.char_to_line(self.start_char as usize + char_idx)
                - (self.start_line_break as usize)
        }
    }

    /// Returns the char index of the start of the given line.
    ///
    /// Note: lines are zero-indexed.
    ///
    /// # Panics
    ///
    /// Panics if `line_idx` is out of bounds (i.e. `line_idx > len_lines()`).
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        // Bounds check
        assert!(
            line_idx <= self.len_lines(),
            "Attempt to index past end of slice: line index {}, slice line length {}",
            line_idx,
            self.len_lines()
        );

        if line_idx == self.len_lines() {
            self.len_chars()
        } else {
            let raw_char_idx = self.node
                .line_to_char(self.start_line_break as usize + line_idx);

            if raw_char_idx < (self.start_char as usize) {
                0
            } else {
                raw_char_idx - self.start_char as usize
            }
        }
    }

    //-----------------------------------------------------------------------
    // Fetch methods

    /// Returns the char at `char_idx`.
    ///
    /// # Panics
    ///
    /// Panics if `char_idx` is out of bounds (i.e. `char_idx >= len_chars()`).
    pub fn char(&self, char_idx: usize) -> char {
        // Bounds check
        assert!(
            char_idx < self.len_chars(),
            "Attempt to index past end of slice: char index {}, slice char length {}",
            char_idx,
            self.len_chars()
        );

        let (chunk, offset) = self.node
            .get_chunk_at_char(char_idx + self.start_char as usize);
        let byte_idx = char_idx_to_byte_idx(chunk, offset);
        chunk[byte_idx..].chars().nth(0).unwrap()
    }

    /// Returns the line at `line_idx`.
    ///
    /// Note: lines are zero-indexed.
    ///
    /// # Panics
    ///
    /// Panics if `line_idx` is out of bounds (i.e. `line_idx >= len_lines()`).
    pub fn line(&self, line_idx: usize) -> RopeSlice<'a> {
        // Bounds check
        assert!(
            line_idx < self.len_lines(),
            "Attempt to index past end of slice: line index {}, slice line length {}",
            line_idx,
            self.len_lines()
        );

        let start = self.line_to_char(line_idx);
        let end = self.line_to_char(line_idx + 1);

        self.slice(start..end)
    }

    //-----------------------------------------------------------------------
    // Slicing

    /// Returns a sub-slice of the `RopeSlice` in the given char index range.
    ///
    /// Uses range syntax, e.g. `2..7`, `2..`, etc.  The range is in `char`
    /// indices.
    ///
    /// # Panics
    ///
    /// Panics if the start of the range is greater than the end, or the end
    /// is out of bounds (i.e. `end > len_chars()`).
    pub fn slice<R: CharIdxRange>(&self, range: R) -> Self {
        let start = range.start().unwrap_or(0);
        let end = range.end().unwrap_or_else(|| self.len_chars());

        // Bounds check
        assert!(start <= end);
        assert!(
            end <= self.len_chars(),
            "Attempt to slice past end of RopeSlice: slice end {}, RopeSlice length {}",
            end,
            self.len_chars()
        );

        RopeSlice::new_with_range(
            self.node,
            self.start_char as usize + start,
            self.start_char as usize + end,
        )
    }

    //-----------------------------------------------------------------------
    // Iterator methods

    /// Creates an iterator over the bytes of the `RopeSlice`.
    pub fn bytes(&self) -> Bytes<'a> {
        Bytes::new_with_range(self.node, self.start_char as usize, self.end_char as usize)
    }

    /// Creates an iterator over the chars of the `RopeSlice`.
    pub fn chars(&self) -> Chars<'a> {
        Chars::new_with_range(self.node, self.start_char as usize, self.end_char as usize)
    }

    /// Creates an iterator over the lines of the `RopeSlice`.
    pub fn lines(&self) -> Lines<'a> {
        Lines::new_with_range(self.node, self.start_char as usize, self.end_char as usize)
    }

    /// Creates an iterator over the chunks of the `RopeSlice`.
    pub fn chunks(&self) -> Chunks<'a> {
        Chunks::new_with_range(self.node, self.start_char as usize, self.end_char as usize)
    }

    //-----------------------------------------------------------------------
    // Conversion methods

    /// Returns the entire text of the `RopeSlice` as a newly allocated `String`.
    pub fn to_string(&self) -> String {
        let mut text = String::with_capacity(self.len_bytes());
        for chunk in self.chunks() {
            text.push_str(chunk);
        }
        text
    }

    /// Creates a new `Rope` from the contents of the `RopeSlice`.
    pub fn to_rope(&self) -> Rope {
        let mut rope = Rope {
            root: Arc::clone(self.node),
        };

        // Chop off right end if needed
        if self.end_char < self.node.text_info().chars {
            rope.split_off(self.end_char as usize);
        }

        // Chop off left end if needed
        if self.start_char > 0 {
            rope = rope.split_off(self.start_char as usize);
        }

        // Return the rope
        rope
    }
}

//==============================================================

impl<'a> std::fmt::Debug for RopeSlice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_list().entries(self.chunks()).finish()
    }
}

impl<'a> std::fmt::Display for RopeSlice<'a> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for chunk in self.chunks() {
            write!(f, "{}", chunk)?
        }
        Ok(())
    }
}

impl<'a, 'b> std::cmp::PartialEq<RopeSlice<'b>> for RopeSlice<'a> {
    #[inline]
    fn eq(&self, other: &RopeSlice<'b>) -> bool {
        if self.len_bytes() != other.len_bytes() {
            return false;
        }

        let mut chunk_itr_1 = self.chunks();
        let mut chunk_itr_2 = other.chunks();
        let mut chunk1 = chunk_itr_1.next().unwrap_or("");
        let mut chunk2 = chunk_itr_2.next().unwrap_or("");

        loop {
            if chunk1.len() > chunk2.len() {
                if &chunk1[..chunk2.len()] != chunk2 {
                    return false;
                } else {
                    chunk1 = &chunk1[chunk2.len()..];
                    chunk2 = "";
                }
            } else if &chunk2[..chunk1.len()] != chunk1 {
                return false;
            } else {
                chunk2 = &chunk2[chunk1.len()..];
                chunk1 = "";
            }

            if chunk1.is_empty() {
                if let Some(chunk) = chunk_itr_1.next() {
                    chunk1 = chunk;
                } else {
                    break;
                }
            }

            if chunk2.is_empty() {
                if let Some(chunk) = chunk_itr_2.next() {
                    chunk2 = chunk;
                } else {
                    break;
                }
            }
        }

        return true;
    }
}

impl<'a, 'b> std::cmp::PartialEq<&'b str> for RopeSlice<'a> {
    #[inline]
    fn eq(&self, other: &&'b str) -> bool {
        if self.len_bytes() != other.len() {
            return false;
        }

        let mut idx = 0;
        for chunk in self.chunks() {
            if chunk != &other[idx..(idx + chunk.len())] {
                return false;
            }
            idx += chunk.len();
        }

        return true;
    }
}

impl<'a, 'b> std::cmp::PartialEq<RopeSlice<'a>> for &'b str {
    #[inline]
    fn eq(&self, other: &RopeSlice<'a>) -> bool {
        other == self
    }
}

impl<'a> std::cmp::PartialEq<str> for RopeSlice<'a> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        std::cmp::PartialEq::<&str>::eq(self, &other)
    }
}

impl<'a> std::cmp::PartialEq<RopeSlice<'a>> for str {
    #[inline]
    fn eq(&self, other: &RopeSlice<'a>) -> bool {
        std::cmp::PartialEq::<&str>::eq(other, &self)
    }
}

impl<'a> std::cmp::PartialEq<String> for RopeSlice<'a> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl<'a> std::cmp::PartialEq<RopeSlice<'a>> for String {
    #[inline]
    fn eq(&self, other: &RopeSlice<'a>) -> bool {
        self.as_str() == other
    }
}

impl<'a, 'b> std::cmp::PartialEq<std::borrow::Cow<'b, str>> for RopeSlice<'a> {
    #[inline]
    fn eq(&self, other: &std::borrow::Cow<'b, str>) -> bool {
        *self == **other
    }
}

impl<'a, 'b> std::cmp::PartialEq<RopeSlice<'a>> for std::borrow::Cow<'b, str> {
    #[inline]
    fn eq(&self, other: &RopeSlice<'a>) -> bool {
        **self == *other
    }
}

impl<'a> std::cmp::PartialEq<Rope> for RopeSlice<'a> {
    #[inline]
    fn eq(&self, other: &Rope) -> bool {
        *self == other.slice(..)
    }
}

impl<'a> std::cmp::PartialEq<RopeSlice<'a>> for Rope {
    #[inline]
    fn eq(&self, other: &RopeSlice<'a>) -> bool {
        self.slice(..) == *other
    }
}

//===========================================================

/// Trait to generalize over the various `Range` types for `a..b` syntax when
/// expressing char ranges.
pub trait CharIdxRange {
    fn start(&self) -> Option<usize>;
    fn end(&self) -> Option<usize>;
}

impl CharIdxRange for Range<usize> {
    fn start(&self) -> Option<usize> {
        Some(self.start)
    }
    fn end(&self) -> Option<usize> {
        Some(self.end)
    }
}

impl CharIdxRange for RangeTo<usize> {
    fn start(&self) -> Option<usize> {
        None
    }
    fn end(&self) -> Option<usize> {
        Some(self.end)
    }
}

impl CharIdxRange for RangeFrom<usize> {
    fn start(&self) -> Option<usize> {
        Some(self.start)
    }
    fn end(&self) -> Option<usize> {
        None
    }
}

impl CharIdxRange for RangeFull {
    fn start(&self) -> Option<usize> {
        None
    }
    fn end(&self) -> Option<usize> {
        None
    }
}

//===========================================================

#[cfg(test)]
mod tests {
    use Rope;

    // 127 bytes, 103 chars, 1 line
    const TEXT: &str = "Hello there!  How're you doing?  It's \
                        a fine day, isn't it?  Aren't you glad \
                        we're alive?  こんにちは、みんなさん！";
    // 124 bytes, 100 chars, 4 lines
    const TEXT_LINES: &str = "Hello there!  How're you doing?\nIt's \
                              a fine day, isn't it?\nAren't you glad \
                              we're alive?\nこんにちは、みんなさん！";

    #[test]
    fn len_bytes_01() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(7..98);
        assert_eq!(s.len_bytes(), 105);
    }

    #[test]
    fn len_bytes_02() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(43..43);
        assert_eq!(s.len_bytes(), 0);
    }

    #[test]
    fn len_chars_01() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(7..98);
        assert_eq!(s.len_chars(), 91);
    }

    #[test]
    fn len_chars_02() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(43..43);
        assert_eq!(s.len_chars(), 0);
    }

    #[test]
    fn len_lines_01() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..98);
        assert_eq!(s.len_lines(), 3);
    }

    #[test]
    fn len_lines_02() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(43..43);
        assert_eq!(s.len_lines(), 1);
    }

    #[test]
    fn byte_to_char_01() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(88..102);

        // ?  こんにちは、みんなさん

        assert_eq!(0, s.byte_to_char(0));
        assert_eq!(1, s.byte_to_char(1));
        assert_eq!(2, s.byte_to_char(2));

        assert_eq!(3, s.byte_to_char(3));
        assert_eq!(3, s.byte_to_char(4));
        assert_eq!(3, s.byte_to_char(5));

        assert_eq!(4, s.byte_to_char(6));
        assert_eq!(4, s.byte_to_char(7));
        assert_eq!(4, s.byte_to_char(8));

        assert_eq!(13, s.byte_to_char(33));
        assert_eq!(13, s.byte_to_char(34));
        assert_eq!(13, s.byte_to_char(35));
        assert_eq!(14, s.byte_to_char(36));
    }

    #[test]
    fn char_to_byte_01() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(88..102);

        // ?  こんにちは、みんなさん

        assert_eq!(0, s.char_to_byte(0));
        assert_eq!(1, s.char_to_byte(1));
        assert_eq!(2, s.char_to_byte(2));

        assert_eq!(3, s.char_to_byte(3));
        assert_eq!(6, s.char_to_byte(4));
        assert_eq!(33, s.char_to_byte(13));
        assert_eq!(36, s.char_to_byte(14));
    }

    #[test]
    fn char_to_line_01() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..96);

        // 's a fine day, isn't it?\nAren't you glad \
        // we're alive?\nこんにちは、みん

        assert_eq!(0, s.char_to_line(0));
        assert_eq!(0, s.char_to_line(1));

        assert_eq!(0, s.char_to_line(24));
        assert_eq!(1, s.char_to_line(25));
        assert_eq!(1, s.char_to_line(26));

        assert_eq!(1, s.char_to_line(53));
        assert_eq!(2, s.char_to_line(54));
        assert_eq!(2, s.char_to_line(55));

        assert_eq!(3, s.char_to_line(62));
    }

    #[test]
    fn char_to_line_02() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(43..43);

        assert_eq!(1, s.char_to_line(0));
    }

    #[test]
    #[should_panic]
    fn char_to_line_03() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..96);

        s.char_to_line(63);
    }

    #[test]
    fn line_to_char_01() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..96);

        assert_eq!(0, s.line_to_char(0));
        assert_eq!(25, s.line_to_char(1));
        assert_eq!(54, s.line_to_char(2));
        assert_eq!(62, s.line_to_char(3));
    }

    #[test]
    fn line_to_char_02() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(43..43);

        assert_eq!(0, s.line_to_char(0));
        assert_eq!(0, s.line_to_char(1));
    }

    #[test]
    #[should_panic]
    fn line_to_char_03() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..96);

        s.line_to_char(4);
    }

    #[test]
    fn char_01() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(34..100);

        // t's \
        // a fine day, isn't it?  Aren't you glad \
        // we're alive?  こんにちは、みんな

        assert_eq!(s.char(0), 't');
        assert_eq!(s.char(10), ' ');
        assert_eq!(s.char(18), 'n');
        assert_eq!(s.char(65), 'な');
    }

    #[test]
    #[should_panic]
    fn char_02() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(34..100);
        s.char(66);
    }

    #[test]
    #[should_panic]
    fn char_03() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(43..43);
        s.char(0);
    }

    #[test]
    fn line_01() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..96);
        // "'s a fine day, isn't it?\nAren't you glad \
        //  we're alive?\nこんにちは、みん"

        assert_eq!(s.line(0), "'s a fine day, isn't it?\n");
        assert_eq!(s.line(1), "Aren't you glad we're alive?\n");
        assert_eq!(s.line(2), "こんにちは、みん");
    }

    #[test]
    fn line_02() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..59);
        // "'s a fine day, isn't it?\n"

        assert_eq!(s.line(0), "'s a fine day, isn't it?\n");
        assert_eq!(s.line(1), "");
    }

    #[test]
    fn line_03() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(43..43);

        assert_eq!(s.line(0), "");
    }

    #[test]
    #[should_panic]
    fn line_04() {
        let r = Rope::from_str(TEXT_LINES);
        let s = r.slice(34..96);
        s.line(3);
    }

    #[test]
    fn slice_01() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(..);

        let s2 = s1.slice(..);

        assert_eq!(TEXT, s2);
    }

    #[test]
    fn slice_02() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(5..43);

        let s2 = s1.slice(3..25);

        assert_eq!(&TEXT[8..30], s2);
    }

    #[test]
    fn slice_03() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(31..97);

        let s2 = s1.slice(7..64);

        assert_eq!(&TEXT[38..103], s2);
    }

    #[test]
    fn slice_04() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(5..43);

        let s2 = s1.slice(21..21);

        assert_eq!("", s2);
    }

    #[test]
    #[should_panic]
    fn slice_05() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(5..43);

        s.slice(21..20);
    }

    #[test]
    #[should_panic]
    fn slice_06() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(5..43);

        s.slice(37..39);
    }

    #[test]
    fn eq_str_01() {
        let r = Rope::from_str(TEXT);
        let slice = r.slice(..);

        assert_eq!(slice, TEXT);
        assert_eq!(TEXT, slice);
    }

    #[test]
    fn eq_str_02() {
        let r = Rope::from_str(TEXT);
        let slice = r.slice(0..20);

        assert_ne!(slice, TEXT);
        assert_ne!(TEXT, slice);
    }

    #[test]
    fn eq_str_03() {
        let mut r = Rope::from_str(TEXT);
        r.remove(20..21);
        r.insert(20, "z");
        let slice = r.slice(..);

        assert_ne!(slice, TEXT);
        assert_ne!(TEXT, slice);
    }

    #[test]
    fn eq_str_04() {
        let r = Rope::from_str(TEXT);
        let slice = r.slice(..);
        let s: String = TEXT.into();

        assert_eq!(slice, s);
        assert_eq!(s, slice);
    }

    #[test]
    fn eq_rope_slice_01() {
        let r = Rope::from_str(TEXT);
        let s = r.slice(43..43);

        assert_eq!(s, s);
    }

    #[test]
    fn eq_rope_slice_02() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(43..97);
        let s2 = r.slice(43..97);

        assert_eq!(s1, s2);
    }

    #[test]
    fn eq_rope_slice_03() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(43..43);
        let s2 = r.slice(43..45);

        assert_ne!(s1, s2);
    }

    #[test]
    fn eq_rope_slice_04() {
        let r = Rope::from_str(TEXT);
        let s1 = r.slice(43..45);
        let s2 = r.slice(43..43);

        assert_ne!(s1, s2);
    }

    #[test]
    fn eq_rope_slice_05() {
        let r = Rope::from_str("");
        let s = r.slice(0..0);

        assert_eq!(s, s);
    }

    #[test]
    fn to_rope_01() {
        let r1 = Rope::from_str(TEXT);
        let s = r1.slice(..);
        let r2 = s.to_rope();

        assert_eq!(r1, r2);
        assert_eq!(s, r2);
    }

    #[test]
    fn to_rope_02() {
        let r1 = Rope::from_str(TEXT);
        let s = r1.slice(0..24);
        let r2 = s.to_rope();

        assert_eq!(s, r2);
    }

    #[test]
    fn to_rope_03() {
        let r1 = Rope::from_str(TEXT);
        let s = r1.slice(13..89);
        let r2 = s.to_rope();

        assert_eq!(s, r2);
    }

    #[test]
    fn to_rope_04() {
        let r1 = Rope::from_str(TEXT);
        let s = r1.slice(13..41);
        let r2 = s.to_rope();

        assert_eq!(s, r2);
    }

    // Iterator tests are in the iter module
}
