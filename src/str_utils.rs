use std;

/// Uses bit-fiddling magic to count utf8 chars really quickly.
/// We actually count the number of non-starting utf8 bytes, since
/// they have a consistent starting two-bit pattern.  We then
/// subtract from the byte length of the text to get the final
/// count.
#[inline]
pub fn count_chars(text: &str) -> usize {
    const ONEMASK: usize = std::usize::MAX / 0xFF;

    let tsize: usize = std::mem::size_of::<usize>();

    let len = text.len();
    let mut ptr = text.as_ptr();
    let end_ptr = unsafe { ptr.offset(len as isize) };
    let mut inv_count = 0;

    // Take care of any unaligned bytes at the beginning
    let end_pre_ptr = next_aligned_ptr(unsafe { ptr.offset(-1) }, tsize).min(end_ptr);
    while ptr < end_pre_ptr {
        let byte = unsafe { *ptr };
        inv_count += ((byte & 0xC0) == 0x80) as usize;
        ptr = unsafe { ptr.offset(1) };
    }

    // Use usize to count multiple bytes at once, using bit-fiddling magic.
    let mut ptr = ptr as *const usize;
    let end_mid_ptr = (end_ptr as usize - (end_ptr as usize & (tsize - 1))) as *const usize;
    while ptr < end_mid_ptr {
        // Do the clever counting
        let n = unsafe { *ptr };
        let byte_bools = ((n >> 7) & (!n >> 6)) & ONEMASK;
        inv_count += (byte_bools.wrapping_mul(ONEMASK)) >> ((tsize - 1) * 8);
        ptr = unsafe { ptr.offset(1) };
    }

    // Take care of any unaligned bytes at the end
    let mut ptr = ptr as *const u8;
    while ptr < end_ptr {
        let byte = unsafe { *ptr };
        inv_count += ((byte & 0xC0) == 0x80) as usize;
        ptr = unsafe { ptr.offset(1) };
    }

    len - inv_count
}

/// Uses bit-fiddling magic to count line breaks really quickly.
///
/// The following unicode sequences are considered newlines by this function:
/// - u{000A}        (Line Feed)
/// - u{000B}        (Vertical Tab)
/// - u{000C}        (Form Feed)
/// - u{000D}        (Carriage Return)
/// - u{000D}u{000A} (Carriage Return + Line Feed)
/// - u{0085}        (Next Line)
/// - u{2028}        (Line Separator)
/// - u{2029}        (Paragraph Separator)
#[inline]
pub fn count_line_breaks(text: &str) -> usize {
    // TODO: right now this checks the high byte for the large line break codepoints
    // when determining whether to skip the full check.  This penalizes texts that use
    // a lot of code points in those ranges.  We should check the low bytes instead, to
    // better distribute the penalty.
    let tsize: usize = std::mem::size_of::<usize>();

    let len = text.len();
    let mut ptr = text.as_ptr();
    let end_ptr = unsafe { ptr.offset(len as isize) };
    let mut count = 0;

    while ptr < end_ptr {
        // Calculate the next aligned ptr after this one
        let end_aligned_ptr = next_aligned_ptr(ptr, tsize).min(end_ptr);

        // Do the full check, counting as we go.
        while ptr < end_aligned_ptr {
            let byte = unsafe { *ptr };

            // Handle u{000A}, u{000B}, u{000C}, and u{000D}
            if (byte <= 0x0D) && (byte >= 0x0A) {
                // Check for CRLF and go forward one more if it is
                let next = unsafe { ptr.offset(1) };
                if byte == 0x0D && next < end_ptr && unsafe { *next } == 0x0A {
                    ptr = next;
                }

                count += 1;
            }
            // Handle u{0085}
            else if byte == 0xC2 {
                ptr = unsafe { ptr.offset(1) };
                if ptr < end_ptr && unsafe { *ptr } == 0x85 {
                    count += 1;
                }
            }
            // Handle u{2028} and u{2029}
            else if byte == 0xE2 {
                let next1 = unsafe { ptr.offset(1) };
                let next2 = unsafe { ptr.offset(2) };
                if next1 < end_ptr && next2 < end_ptr && unsafe { *next1 } == 0x80 && (unsafe {
                    *next2
                }
                    >> 1)
                    == 0x54
                {
                    count += 1;
                }
                ptr = unsafe { ptr.offset(2) };
            }

            ptr = unsafe { ptr.offset(1) };
        }

        // Use usize to to check if it's even possible that there are any
        // line endings, using bit-fiddling magic.
        if ptr == end_aligned_ptr {
            while unsafe { ptr.offset(tsize as isize) } < end_ptr {
                let n = unsafe { *(ptr as *const usize) };

                // If there's a possibility that there might be a line-ending, stop
                // and do the full check.
                if has_bytes_less_than(n, 0x0E) || has_byte(n, 0xC2) || has_byte(n, 0xE2) {
                    break;
                }

                ptr = unsafe { ptr.offset(tsize as isize) };
            }
        }
    }

    count
}

#[inline]
pub fn byte_idx_to_char_idx(text: &str, byte_idx: usize) -> usize {
    if byte_idx == 0 {
        return 0;
    } else if byte_idx >= text.len() {
        return count_chars(text);
    } else {
        return count_chars(unsafe {
            std::str::from_utf8_unchecked(&text.as_bytes()[0..(byte_idx + 1)])
        }) - 1;
    }
}

#[inline]
pub fn byte_idx_to_line_idx(text: &str, byte_idx: usize) -> usize {
    let mut line_i = 1;
    for offset in LineBreakIter::new(text) {
        if byte_idx < offset {
            break;
        } else {
            line_i += 1;
        }
    }
    line_i - 1
}

#[inline]
pub fn char_idx_to_byte_idx(text: &str, char_idx: usize) -> usize {
    const ONEMASK: usize = std::usize::MAX / 0xFF;
    let tsize: usize = std::mem::size_of::<usize>();

    let mut char_count = 0;
    let mut ptr = text.as_ptr();
    let start_ptr = text.as_ptr();
    let end_ptr = unsafe { ptr.offset(text.len() as isize) };

    // Take care of any unaligned bytes at the beginning
    let end_pre_ptr = {
        let aligned = ptr as usize + (tsize - (ptr as usize & (tsize - 1)));
        (end_ptr as usize).min(aligned) as *const u8
    };
    while ptr < end_pre_ptr && char_count <= char_idx {
        let byte = unsafe { *ptr };
        char_count += ((byte & 0xC0) != 0x80) as usize;
        ptr = unsafe { ptr.offset(1) };
    }

    // Use usize to count multiple bytes at once, using bit-fiddling magic.
    let mut ptr = ptr as *const usize;
    let end_mid_ptr = (end_ptr as usize - (end_ptr as usize & (tsize - 1))) as *const usize;
    while ptr < end_mid_ptr && (char_count + tsize) <= char_idx {
        // Do the clever counting
        let n = unsafe { *ptr };
        let byte_bools = (!((n >> 7) & (!n >> 6))) & ONEMASK;
        char_count += (byte_bools.wrapping_mul(ONEMASK)) >> ((tsize - 1) * 8);
        ptr = unsafe { ptr.offset(1) };
    }

    // Take care of any unaligned bytes at the end
    let mut ptr = ptr as *const u8;
    while ptr < end_ptr && char_count <= char_idx {
        let byte = unsafe { *ptr };
        char_count += ((byte & 0xC0) != 0x80) as usize;
        ptr = unsafe { ptr.offset(1) };
    }

    // Finish up
    let byte_count = ptr as usize - start_ptr as usize;
    if ptr == end_ptr && char_count == char_idx {
        byte_count
    } else {
        byte_count - 1
    }
}

#[inline]
pub fn char_idx_to_line_idx(text: &str, char_idx: usize) -> usize {
    byte_idx_to_line_idx(text, char_idx_to_byte_idx(text, char_idx))
}

#[inline]
pub fn line_idx_to_byte_idx(text: &str, line_idx: usize) -> usize {
    if line_idx == 0 {
        0
    } else {
        LineBreakIter::new(text)
            .nth(line_idx - 1)
            .unwrap_or_else(|| text.len())
    }
}

#[inline]
pub fn line_idx_to_char_idx(text: &str, line_idx: usize) -> usize {
    byte_idx_to_char_idx(text, line_idx_to_byte_idx(text, line_idx))
}

#[inline(always)]
pub fn has_bytes_less_than(word: usize, n: u8) -> bool {
    const ONEMASK: usize = std::usize::MAX / 0xFF;
    ((word.wrapping_sub(ONEMASK * n as usize)) & !word & (ONEMASK * 128)) != 0
}

#[inline(always)]
pub fn has_byte(word: usize, n: u8) -> bool {
    const ONEMASK_LOW: usize = std::usize::MAX / 0xFF;
    const ONEMASK_HIGH: usize = ONEMASK_LOW << 7;
    let word = word ^ (n as usize * ONEMASK_LOW);
    (word.wrapping_sub(ONEMASK_LOW) & !word & ONEMASK_HIGH) != 0
}

#[inline]
pub fn next_aligned_ptr<T>(ptr: *const T, alignment: usize) -> *const T {
    (ptr as usize + (alignment - (ptr as usize & (alignment - 1)))) as *const T
}

//======================================================================

/// An iterator that yields the byte indices of line breaks in a string.
/// A line break in this case is the point immediately *after* a newline
/// character.
///
/// The following unicode sequences are considered newlines by this function:
/// - u{000A}        (Line Feed)
/// - u{000B}        (Vertical Tab)
/// - u{000C}        (Form Feed)
/// - u{000D}        (Carriage Return)
/// - u{000D}u{000A} (Carriage Return + Line Feed)
/// - u{0085}        (Next Line)
/// - u{2028}        (Line Separator)
/// - u{2029}        (Paragraph Separator)
pub(crate) struct LineBreakIter<'a> {
    byte_itr: std::str::Bytes<'a>,
    byte_idx: usize,
}

impl<'a> LineBreakIter<'a> {
    #[inline]
    pub fn new(text: &str) -> LineBreakIter {
        LineBreakIter {
            byte_itr: text.bytes(),
            byte_idx: 0,
        }
    }
}

impl<'a> Iterator for LineBreakIter<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        while let Some(byte) = self.byte_itr.next() {
            self.byte_idx += 1;
            // Handle u{000A}, u{000B}, u{000C}, and u{000D}
            if (byte <= 0x0D) && (byte >= 0x0A) {
                if byte == 0x0D {
                    // We're basically "peeking" here.
                    if let Some(0x0A) = self.byte_itr.clone().next() {
                        self.byte_itr.next();
                        self.byte_idx += 1;
                    }
                }
                return Some(self.byte_idx);
            }
            // Handle u{0085}
            else if byte == 0xC2 {
                self.byte_idx += 1;
                if let Some(0x85) = self.byte_itr.next() {
                    return Some(self.byte_idx);
                }
            }
            // Handle u{2028} and u{2029}
            else if byte == 0xE2 {
                self.byte_idx += 2;
                let byte2 = self.byte_itr.next().unwrap();
                let byte3 = self.byte_itr.next().unwrap() >> 1;
                if byte2 == 0x80 && byte3 == 0x54 {
                    return Some(self.byte_idx);
                }
            }
        }

        return None;
    }
}

//======================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_chars_01() {
        let text =
            "Hello せかい! Hello せかい! Hello せかい! Hello せかい! Hello せかい!";

        assert_eq!(54, count_chars(text));
    }

    #[test]
    fn line_breaks_iter_01() {
        let text = "\u{000A}Hello\u{000D}\u{000A}\u{000D}せ\u{000B}か\u{000C}い\u{0085}. \
                    There\u{2028}is something.\u{2029}";
        let mut itr = LineBreakIter::new(text);
        assert_eq!(48, text.len());
        assert_eq!(Some(1), itr.next());
        assert_eq!(Some(8), itr.next());
        assert_eq!(Some(9), itr.next());
        assert_eq!(Some(13), itr.next());
        assert_eq!(Some(17), itr.next());
        assert_eq!(Some(22), itr.next());
        assert_eq!(Some(32), itr.next());
        assert_eq!(Some(48), itr.next());
        assert_eq!(None, itr.next());
    }

    #[test]
    fn count_line_breaks_01() {
        let text = "\u{000A}Hello\u{000D}\u{000A}\u{000D}せ\u{000B}か\u{000C}い\u{0085}. \
                    There\u{2028}is something.\u{2029}";
        assert_eq!(48, text.len());
        assert_eq!(8, count_line_breaks(text));
    }

    #[test]
    fn count_line_breaks_02() {
        let text = "\u{000A}Hello world!  This is a longer text.\u{000D}\u{000A}\u{000D}To better test that skipping by usize doesn't mess things up.\u{000B}Hello せかい!\u{000C}\u{0085}Yet more text.  How boring.\u{2028}Hi.\u{2029}\u{000A}Hello world!  This is a longer text.\u{000D}\u{000A}\u{000D}To better test that skipping by usize doesn't mess things up.\u{000B}Hello せかい!\u{000C}\u{0085}Yet more text.  How boring.\u{2028}Hi.\u{2029}\u{000A}Hello world!  This is a longer text.\u{000D}\u{000A}\u{000D}To better test that skipping by usize doesn't mess things up.\u{000B}Hello せかい!\u{000C}\u{0085}Yet more text.  How boring.\u{2028}Hi.\u{2029}\u{000A}Hello world!  This is a longer text.\u{000D}\u{000A}\u{000D}To better test that skipping by usize doesn't mess things up.\u{000B}Hello せかい!\u{000C}\u{0085}Yet more text.  How boring.\u{2028}Hi.\u{2029}";
        assert_eq!(count_line_breaks(text), LineBreakIter::new(text).count());
    }

    #[test]
    fn byte_idx_to_char_idx_01() {
        let text = "Hello せかい!";
        assert_eq!(0, byte_idx_to_char_idx(text, 0));
        assert_eq!(1, byte_idx_to_char_idx(text, 1));
        assert_eq!(8, byte_idx_to_char_idx(text, 12));
        assert_eq!(10, byte_idx_to_char_idx(text, 16));
    }

    #[test]
    fn byte_idx_to_char_idx_02() {
        let text = "せかい";
        assert_eq!(0, byte_idx_to_char_idx(text, 0));
        assert_eq!(0, byte_idx_to_char_idx(text, 1));
        assert_eq!(0, byte_idx_to_char_idx(text, 2));
        assert_eq!(1, byte_idx_to_char_idx(text, 3));
        assert_eq!(1, byte_idx_to_char_idx(text, 4));
        assert_eq!(1, byte_idx_to_char_idx(text, 5));
        assert_eq!(2, byte_idx_to_char_idx(text, 6));
        assert_eq!(2, byte_idx_to_char_idx(text, 7));
        assert_eq!(2, byte_idx_to_char_idx(text, 8));
        assert_eq!(3, byte_idx_to_char_idx(text, 9));
    }

    #[test]
    fn byte_idx_to_line_idx_01() {
        let text = "Here\nare\nsome\nwords";
        assert_eq!(0, byte_idx_to_line_idx(text, 0));
        assert_eq!(0, byte_idx_to_line_idx(text, 4));
        assert_eq!(1, byte_idx_to_line_idx(text, 5));
        assert_eq!(1, byte_idx_to_line_idx(text, 8));
        assert_eq!(2, byte_idx_to_line_idx(text, 9));
        assert_eq!(2, byte_idx_to_line_idx(text, 13));
        assert_eq!(3, byte_idx_to_line_idx(text, 14));
        assert_eq!(3, byte_idx_to_line_idx(text, 19));
    }

    #[test]
    fn byte_idx_to_line_idx_02() {
        let text = "\nHere\nare\nsome\nwords\n";
        assert_eq!(0, byte_idx_to_line_idx(text, 0));
        assert_eq!(1, byte_idx_to_line_idx(text, 1));
        assert_eq!(1, byte_idx_to_line_idx(text, 5));
        assert_eq!(2, byte_idx_to_line_idx(text, 6));
        assert_eq!(2, byte_idx_to_line_idx(text, 9));
        assert_eq!(3, byte_idx_to_line_idx(text, 10));
        assert_eq!(3, byte_idx_to_line_idx(text, 14));
        assert_eq!(4, byte_idx_to_line_idx(text, 15));
        assert_eq!(4, byte_idx_to_line_idx(text, 20));
        assert_eq!(5, byte_idx_to_line_idx(text, 21));
    }

    #[test]
    fn byte_idx_to_line_idx_03() {
        let text = "Here\r\nare\r\nsome\r\nwords";
        assert_eq!(0, byte_idx_to_line_idx(text, 0));
        assert_eq!(0, byte_idx_to_line_idx(text, 4));
        assert_eq!(0, byte_idx_to_line_idx(text, 5));
        assert_eq!(1, byte_idx_to_line_idx(text, 6));
        assert_eq!(1, byte_idx_to_line_idx(text, 9));
        assert_eq!(1, byte_idx_to_line_idx(text, 10));
        assert_eq!(2, byte_idx_to_line_idx(text, 11));
        assert_eq!(2, byte_idx_to_line_idx(text, 15));
        assert_eq!(2, byte_idx_to_line_idx(text, 16));
        assert_eq!(3, byte_idx_to_line_idx(text, 17));
    }

    #[test]
    fn char_idx_to_byte_idx_01() {
        let text = "Hello せかい!";
        assert_eq!(0, char_idx_to_byte_idx(text, 0));
        assert_eq!(1, char_idx_to_byte_idx(text, 1));
        assert_eq!(2, char_idx_to_byte_idx(text, 2));
        assert_eq!(5, char_idx_to_byte_idx(text, 5));
        assert_eq!(6, char_idx_to_byte_idx(text, 6));
        assert_eq!(12, char_idx_to_byte_idx(text, 8));
        assert_eq!(15, char_idx_to_byte_idx(text, 9));
        assert_eq!(16, char_idx_to_byte_idx(text, 10));
    }

    #[test]
    fn char_idx_to_byte_idx_02() {
        let text = "せかい";
        assert_eq!(0, char_idx_to_byte_idx(text, 0));
        assert_eq!(3, char_idx_to_byte_idx(text, 1));
        assert_eq!(6, char_idx_to_byte_idx(text, 2));
        assert_eq!(9, char_idx_to_byte_idx(text, 3));
    }

    #[test]
    fn char_idx_to_byte_idx_03() {
        let text = "Hello world!";
        assert_eq!(0, char_idx_to_byte_idx(text, 0));
        assert_eq!(1, char_idx_to_byte_idx(text, 1));
        assert_eq!(8, char_idx_to_byte_idx(text, 8));
        assert_eq!(11, char_idx_to_byte_idx(text, 11));
        assert_eq!(12, char_idx_to_byte_idx(text, 12));
    }

    #[test]
    fn char_idx_to_byte_idx_04() {
        let text = "Hello world! Hello せかい! Hello world! Hello せかい! \
                    Hello world! Hello せかい! Hello world! Hello せかい! \
                    Hello world! Hello せかい! Hello world! Hello せかい! \
                    Hello world! Hello せかい! Hello world! Hello せかい!";
        assert_eq!(0, char_idx_to_byte_idx(text, 0));
        assert_eq!(30, char_idx_to_byte_idx(text, 24));
        assert_eq!(60, char_idx_to_byte_idx(text, 48));
        assert_eq!(90, char_idx_to_byte_idx(text, 72));
        assert_eq!(115, char_idx_to_byte_idx(text, 93));
        assert_eq!(120, char_idx_to_byte_idx(text, 96));
        assert_eq!(150, char_idx_to_byte_idx(text, 120));
        assert_eq!(180, char_idx_to_byte_idx(text, 144));
        assert_eq!(210, char_idx_to_byte_idx(text, 168));
        assert_eq!(239, char_idx_to_byte_idx(text, 191));
    }

    #[test]
    fn char_idx_to_line_idx_01() {
        let text = "Hello せ\nか\nい!";
        assert_eq!(0, char_idx_to_line_idx(text, 0));
        assert_eq!(0, char_idx_to_line_idx(text, 7));
        assert_eq!(1, char_idx_to_line_idx(text, 8));
        assert_eq!(1, char_idx_to_line_idx(text, 9));
        assert_eq!(2, char_idx_to_line_idx(text, 10));
    }

    #[test]
    fn line_idx_to_byte_idx_01() {
        let text = "Here\r\nare\r\nsome\r\nwords";
        assert_eq!(0, line_idx_to_byte_idx(text, 0));
        assert_eq!(6, line_idx_to_byte_idx(text, 1));
        assert_eq!(11, line_idx_to_byte_idx(text, 2));
        assert_eq!(17, line_idx_to_byte_idx(text, 3));
    }

    #[test]
    fn line_idx_to_byte_idx_02() {
        let text = "\nHere\nare\nsome\nwords\n";
        assert_eq!(0, line_idx_to_byte_idx(text, 0));
        assert_eq!(1, line_idx_to_byte_idx(text, 1));
        assert_eq!(6, line_idx_to_byte_idx(text, 2));
        assert_eq!(10, line_idx_to_byte_idx(text, 3));
        assert_eq!(15, line_idx_to_byte_idx(text, 4));
        assert_eq!(21, line_idx_to_byte_idx(text, 5));
    }

    #[test]
    fn line_idx_to_char_idx_01() {
        let text = "Hello せ\nか\nい!";
        assert_eq!(0, line_idx_to_char_idx(text, 0));
        assert_eq!(8, line_idx_to_char_idx(text, 1));
        assert_eq!(10, line_idx_to_char_idx(text, 2));
    }

    #[test]
    fn line_byte_round_trip() {
        let text = "\nHere\nare\nsome\nwords\n";
        assert_eq!(6, line_idx_to_byte_idx(text, byte_idx_to_line_idx(text, 6)));
        assert_eq!(2, byte_idx_to_line_idx(text, line_idx_to_byte_idx(text, 2)));

        assert_eq!(0, line_idx_to_byte_idx(text, byte_idx_to_line_idx(text, 0)));
        assert_eq!(0, byte_idx_to_line_idx(text, line_idx_to_byte_idx(text, 0)));

        assert_eq!(
            21,
            line_idx_to_byte_idx(text, byte_idx_to_line_idx(text, 21))
        );
        assert_eq!(5, byte_idx_to_line_idx(text, line_idx_to_byte_idx(text, 5)));
    }

    #[test]
    fn line_char_round_trip() {
        let text = "\nHere\nare\nsome\nwords\n";
        assert_eq!(6, line_idx_to_char_idx(text, char_idx_to_line_idx(text, 6)));
        assert_eq!(2, char_idx_to_line_idx(text, line_idx_to_char_idx(text, 2)));

        assert_eq!(0, line_idx_to_char_idx(text, char_idx_to_line_idx(text, 0)));
        assert_eq!(0, char_idx_to_line_idx(text, line_idx_to_char_idx(text, 0)));

        assert_eq!(
            21,
            line_idx_to_char_idx(text, char_idx_to_line_idx(text, 21))
        );
        assert_eq!(5, char_idx_to_line_idx(text, line_idx_to_char_idx(text, 5)));
    }

    #[test]
    fn has_bytes_less_than_01() {
        let v = 0x0709080905090609;
        assert!(has_bytes_less_than(v, 0x0A));
        assert!(has_bytes_less_than(v, 0x06));
        assert!(!has_bytes_less_than(v, 0x05));
    }

    #[test]
    fn has_byte_01() {
        let v = 0x070908A60509E209;
        assert!(has_byte(v, 0x07));
        assert!(has_byte(v, 0x09));
        assert!(has_byte(v, 0x08));
        assert!(has_byte(v, 0xA6));
        assert!(has_byte(v, 0x05));
        assert!(has_byte(v, 0xE2));

        assert!(!has_byte(v, 0xA0));
        assert!(!has_byte(v, 0xA7));
        assert!(!has_byte(v, 0x06));
        assert!(!has_byte(v, 0xE3));
    }
}
