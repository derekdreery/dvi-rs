//! Parsers for each instruction type

use crate::{util::parse_223, FontDef, Instruction};

use nom::{
    bytes::streaming::take,
    combinator::map,
    number::streaming::{be_i16, be_i24, be_i32, be_i8, be_u16, be_u24, be_u32, be_u8},
    IResult, Needed,
};

pub fn parse(i: &[u8]) -> IResult<&[u8], Instruction> {
    match i.get(0) {
        Some(&op) if op <= 127 => Ok((&i[1..], Instruction::Set(op as u32))),
        Some(&op) if op >= 171 && op <= 234 => Ok((&i[1..], Instruction::Font((op - 171) as u32))),
        Some(&_) => parse_complex(i),
        None => Err(nom::Err::Incomplete(Needed::Unknown)),
    }
}

fn parse_complex(input: &[u8]) -> IResult<&[u8], Instruction> {
    let (input, code) = be_u8(input)?;
    match code {
        // Set
        128 => map(be_u8, |ch| Instruction::Set(ch.into()))(input),
        129 => map(be_u16, |ch| Instruction::Set(ch.into()))(input),
        130 => map(be_u24, Instruction::Set)(input),
        131 => map(be_u32, Instruction::Set)(input),
        // SetRule
        132 => {
            let (input, a) = be_i32(input)?;
            let (input, b) = be_i32(input)?;
            Ok((input, Instruction::SetRule(a, b)))
        }
        // Put
        133 => map(be_u8, |ch| Instruction::Put(ch.into()))(input),
        134 => map(be_u16, |ch| Instruction::Put(ch.into()))(input),
        135 => map(be_u24, Instruction::Put)(input),
        136 => map(be_u32, Instruction::Put)(input),
        // SetRule
        137 => {
            let (input, a) = be_i32(input)?;
            let (input, b) = be_i32(input)?;
            Ok((input, Instruction::PutRule(a, b)))
        }
        // Nop
        138 => Ok((input, Instruction::Nop)),
        // Bop
        139 => {
            let (input, c0) = be_i32(input)?;
            let (input, c1) = be_i32(input)?;
            let (input, c2) = be_i32(input)?;
            let (input, c3) = be_i32(input)?;
            let (input, c4) = be_i32(input)?;
            let (input, c5) = be_i32(input)?;
            let (input, c6) = be_i32(input)?;
            let (input, c7) = be_i32(input)?;
            let (input, c8) = be_i32(input)?;
            let (input, c9) = be_i32(input)?;
            let (input, p) = be_i32(input)?;
            Ok((
                input,
                Instruction::Bop([c0, c1, c2, c3, c4, c5, c6, c7, c8, c9], p),
            ))
        }
        // Eop
        140 => Ok((input, Instruction::Eop)),
        // Push
        141 => Ok((input, Instruction::Push)),
        // Pop
        142 => Ok((input, Instruction::Pop)),
        // Right
        143 => map(be_i8, |a| Instruction::Right(a.into()))(input),
        144 => map(be_i16, |a| Instruction::Right(a.into()))(input),
        145 => map(be_i24, Instruction::Right)(input),
        146 => map(be_i32, Instruction::Right)(input),
        // W
        147 => Ok((input, Instruction::W(None))),
        148 => map(be_i8, |a| Instruction::W(Some(a.into())))(input),
        149 => map(be_i16, |a| Instruction::W(Some(a.into())))(input),
        150 => map(be_i24, |a| Instruction::W(Some(a)))(input),
        151 => map(be_i32, |a| Instruction::W(Some(a)))(input),
        // X
        152 => Ok((input, Instruction::X(None))),
        153 => map(be_i8, |a| Instruction::X(Some(a.into())))(input),
        154 => map(be_i16, |a| Instruction::X(Some(a.into())))(input),
        155 => map(be_i24, |a| Instruction::X(Some(a)))(input),
        156 => map(be_i32, |a| Instruction::X(Some(a)))(input),
        // Down
        157 => map(be_i8, |a| Instruction::Down(a.into()))(input),
        158 => map(be_i16, |a| Instruction::Down(a.into()))(input),
        159 => map(be_i24, Instruction::Down)(input),
        160 => map(be_i32, Instruction::Down)(input),
        // Y
        161 => Ok((input, Instruction::Y(None))),
        162 => map(be_i8, |a| Instruction::Y(Some(a.into())))(input),
        163 => map(be_i16, |a| Instruction::Y(Some(a.into())))(input),
        164 => map(be_i24, |a| Instruction::Y(Some(a)))(input),
        165 => map(be_i32, |a| Instruction::Y(Some(a)))(input),
        // Z
        166 => Ok((input, Instruction::Z(None))),
        167 => map(be_i8, |a| Instruction::Z(Some(a.into())))(input),
        168 => map(be_i16, |a| Instruction::Z(Some(a.into())))(input),
        169 => map(be_i24, |a| Instruction::Z(Some(a)))(input),
        170 => map(be_i32, |a| Instruction::Z(Some(a)))(input),
        // Font
        235 => map(be_u8, |f| Instruction::Font(f.into()))(input),
        236 => map(be_u16, |f| Instruction::Font(f.into()))(input),
        237 => map(be_u24, Instruction::Font)(input),
        238 => map(be_u32, Instruction::Font)(input),
        // Xxx
        239 => {
            let (input, length) = be_u8(input)?;
            let (input, slice) = take(length)(input)?;
            Ok((input, Instruction::Xxx(slice.to_owned())))
        }
        240 => {
            let (input, length) = be_u16(input)?;
            let (input, slice) = take(length)(input)?;
            Ok((input, Instruction::Xxx(slice.to_owned())))
        }
        241 => {
            let (input, length) = be_u24(input)?;
            let (input, slice) = take(length)(input)?;
            Ok((input, Instruction::Xxx(slice.to_owned())))
        }
        242 => {
            let (input, length) = be_u32(input)?;
            let (input, slice) = take(length)(input)?;
            Ok((input, Instruction::Xxx(slice.to_owned())))
        }
        // FontDef
        243 => {
            let (input, number) = be_u8(input)?;
            font_def(input, number.into())
        }
        244 => {
            let (input, number) = be_u16(input)?;
            font_def(input, number.into())
        }
        245 => {
            let (input, number) = be_u24(input)?;
            font_def(input, number)
        }
        246 => {
            let (input, number) = be_u32(input)?;
            font_def(input, number)
        }
        // Pre
        247 => {
            let (input, format) = be_u8(input)?;
            let (input, numerator) = be_u32(input)?;
            let (input, denominator) = be_u32(input)?;
            let (input, magnification) = be_u32(input)?;
            let (input, comment_length) = be_u8(input)?;
            let (input, comment) = take(comment_length)(input)?;
            Ok((
                input,
                Instruction::Pre {
                    format,
                    numerator,
                    denominator,
                    magnification,
                    comment: comment.to_owned(),
                },
            ))
        }
        248 => {
            let (input, final_bop_pointer) = be_i32(input)?;
            let (input, numerator) = be_u32(input)?;
            let (input, denominator) = be_u32(input)?;
            let (input, magnification) = be_u32(input)?;
            let (input, tallest_height) = be_i32(input)?;
            let (input, widest_width) = be_i32(input)?;
            let (input, max_stack_depth) = be_u16(input)?;
            let (input, total_no_pages) = be_u16(input)?;
            Ok((
                input,
                Instruction::Post {
                    final_bop_pointer,
                    numerator,
                    denominator,
                    magnification,
                    tallest_height,
                    widest_width,
                    max_stack_depth,
                    total_no_pages,
                },
            ))
        }
        249 => {
            let (input, post_pointer) = be_u32(input)?;
            let (input, ident) = be_u8(input)?;
            let (input, two_two_three) = parse_223(input)?;
            Ok((
                input,
                Instruction::PostPost {
                    post_pointer,
                    ident,
                    two_two_three,
                },
            ))
        }
        _ => unreachable!(),
    }
}

fn font_def(input: &[u8], number: u32) -> IResult<&[u8], Instruction> {
    let (input, checksum) = be_u32(input)?;
    let (input, scale_factor) = be_u32(input)?;
    let (input, design_size) = be_u32(input)?;
    let (input, directory_len) = be_u8(input)?;
    let (input, filename_len) = be_u8(input)?;
    let (input, directory) = take(directory_len)(input)?;
    let (input, filename) = take(filename_len)(input)?;
    let directory = match directory_len {
        0 => None,
        _ => Some(directory.to_owned()),
    };
    Ok((
        input,
        Instruction::FontDef(FontDef {
            number,
            checksum,
            scale_factor,
            design_size,
            directory,
            filename: filename.to_owned(),
        }),
    ))
}
