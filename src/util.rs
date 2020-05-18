use nom::IResult;

/// Get the number of bytes required to store a u32
pub(crate) fn byte_width(val: u32) -> u8 {
    if (val & !0xffu32) == 0 {
        1
    } else if (val & !0xff_ffu32) == 0 {
        2
    } else if (val & !0xff_ff_ffu32) == 0 {
        3
    } else {
        4
    }
}

/// Get the number of bytes required to store an i32
pub(crate) fn byte_width_signed(val: i32) -> u8 {
    // TODO the logic of this could do with looking over - I will remove this message once I can
    // parse documents reliably
    // TODO benchmark against < u8::MAX etc
    if val as u32 & !0x7f == 0 || !val as u32 & !0x7f == 0 {
        1
    } else if val as u32 & !0x7f_ff == 0 || !val as u32 & !0x7f_ff == 0 {
        2
    } else if val as u32 & !0x7f_ff_ff == 0 || !val as u32 & !0x7f_ff_ff == 0 {
        3
    } else {
        4
    }
}

/// A parser to count the number of times the byte 223 occurs. This parser converts Incomplete to
/// Done, which is normally bad, but since these trailing bytes make no difference to the semantic
/// meaning of the document, we don't care if we haven't read them all yet.
pub(crate) fn parse_223(i: &[u8]) -> IResult<&[u8], u32> {
    let mut i = i;
    let mut count = 0;
    while let IResult::Done(inext, _) = tag!(i, &[223][..]) {
        count += 1;
        i = inext;
    }
    IResult::Done(i, count)
}

#[cfg(test)]
mod tests {

    #[test]
    fn byte_width() {
        use super::byte_width;
        assert_eq!(byte_width(0x7f_ff_ff_ff), 4);
        assert_eq!(byte_width(0x7f_ff_ff), 3);
        assert_eq!(byte_width(0x7f_ff), 2);
        assert_eq!(byte_width(0x7f), 1);
        assert_eq!(byte_width(0), 1);
    }

    #[test]
    fn byte_width_signed() {
        use super::byte_width_signed;
        assert_eq!(byte_width_signed(0), 1);
        assert_eq!(byte_width_signed(0x7f), 1);
        assert_eq!(byte_width_signed(0x7f_ff), 2);
        assert_eq!(byte_width_signed(0x7f_ff_ff), 3);
        assert_eq!(byte_width_signed(0x7f_ff_ff_ff), 4);
        assert_eq!(byte_width_signed(-0x7f), 1);
        assert_eq!(byte_width_signed(-0x7f_ff), 2);
        assert_eq!(byte_width_signed(-0x7f_ff_ff), 3);
        assert_eq!(byte_width_signed(-0x7f_ff_ff_ff), 4);
        assert_eq!(byte_width_signed(::std::i32::MAX), 4);
        assert_eq!(byte_width_signed(::std::i32::MIN), 4);
    }
}
