//! Parsers for each instruction type

use super::util::parse_223;
use super::{FontDef, Instruction};

use nom::{be_i16, be_i24, be_i32, be_i8, be_u16, be_u24, be_u32, be_u8, Err, IResult, Needed};

pub fn parse(i: &[u8]) -> IResult<&[u8], Instruction> {
    match i.get(0) {
        Some(&op) if op <= 127 => Ok((&i[1..], Instruction::Set(op as u32))),
        Some(&op) if op >= 171 && op <= 234 => Ok((&i[1..], Instruction::Font((op - 171) as u32))),
        Some(&_) => parse_complex(i),
        None => Err(Err::Incomplete(Needed::Unknown)),
    }
}

named!(
    parse_complex<Instruction>,
    switch!(be_u8,
        // Set
        128 => map!(be_u8, |ch| Instruction::Set(ch as u32)) |
        129 => map!(be_u16, |ch| Instruction::Set(ch as u32)) |
        130 => map!(be_u24, |ch| Instruction::Set(ch)) |
        131 => map!(be_u32, |ch| Instruction::Set(ch)) |
        // SetRule
        132 => do_parse!(
            a: be_i32 >>
            b: be_i32 >>
            (Instruction::SetRule(a, b))
        ) |
        // Put
        133 => map!(be_u8, |ch| Instruction::Put(ch as u32)) |
        134 => map!(be_u16, |ch| Instruction::Put(ch as u32)) |
        135 => map!(be_u24, |ch| Instruction::Put(ch)) |
        136 => map!(be_u32, |ch| Instruction::Put(ch)) |
        // SetRule
        137 => do_parse!(
            a: be_i32 >>
            b: be_i32 >>
            (Instruction::PutRule(a, b))
        ) |
        // Nop
        138 => value!(Instruction::Nop) |
        // Bop
        139 => do_parse!(
            c0: be_i32 >>
            c1: be_i32 >>
            c2: be_i32 >>
            c3: be_i32 >>
            c4: be_i32 >>
            c5: be_i32 >>
            c6: be_i32 >>
            c7: be_i32 >>
            c8: be_i32 >>
            c9: be_i32 >>
            p: be_i32 >>
            (Instruction::Bop([c0, c1, c2, c3, c4, c5, c6, c7, c8, c9], p))
        ) |
        // Eop
        140 => value!(Instruction::Eop) |
        // Push
        141 => value!(Instruction::Push) |
        // Pop
        142 => value!(Instruction::Pop) |
        // Right
        143 => map!(be_i8, |a| Instruction::Right(a as i32)) |
        144 => map!(be_i16, |a| Instruction::Right(a as i32)) |
        145 => map!(be_i24, |a| Instruction::Right(a)) |
        146 => map!(be_i32, |a| Instruction::Right(a)) |
        // W
        147 => value!(Instruction::W(None)) |
        148 => map!(be_i8, |a| Instruction::W(Some(a as i32))) |
        149 => map!(be_i16, |a| Instruction::W(Some(a as i32))) |
        150 => map!(be_i24, |a| Instruction::W(Some(a))) |
        151 => map!(be_i32, |a| Instruction::W(Some(a))) |
        // X
        152 => value!(Instruction::X(None)) |
        153 => map!(be_i8, |a| Instruction::X(Some(a as i32))) |
        154 => map!(be_i16, |a| Instruction::X(Some(a as i32))) |
        155 => map!(be_i24, |a| Instruction::X(Some(a))) |
        156 => map!(be_i32, |a| Instruction::X(Some(a))) |
        // Down
        157 => map!(be_i8, |a| Instruction::Down(a as i32)) |
        158 => map!(be_i16, |a| Instruction::Down(a as i32)) |
        159 => map!(be_i24, |a| Instruction::Down(a)) |
        160 => map!(be_i32, |a| Instruction::Down(a)) |
        // Y
        161 => value!(Instruction::Y(None)) |
        162 => map!(be_i8, |a| Instruction::Y(Some(a as i32))) |
        163 => map!(be_i16, |a| Instruction::Y(Some(a as i32))) |
        164 => map!(be_i24, |a| Instruction::Y(Some(a))) |
        165 => map!(be_i32, |a| Instruction::Y(Some(a))) |
        // Z
        166 => value!(Instruction::Z(None)) |
        167 => map!(be_i8, |a| Instruction::Z(Some(a as i32))) |
        168 => map!(be_i16, |a| Instruction::Z(Some(a as i32))) |
        169 => map!(be_i24, |a| Instruction::Z(Some(a))) |
        170 => map!(be_i32, |a| Instruction::Z(Some(a))) |
        // Font
        235 => map!(be_u8, |f| Instruction::Font(f as u32)) |
        236 => map!(be_u16, |f| Instruction::Font(f as u32)) |
        237 => map!(be_u24, |f| Instruction::Font(f)) |
        238 => map!(be_u32, |f| Instruction::Font(f)) |
        // Xxx
        239 => do_parse!(
            length: be_u8 >>
            slice: take!(length) >>
            (Instruction::Xxx(slice.to_owned()))
        ) |
        240 => do_parse!(
            length: be_u16 >>
            slice: take!(length) >>
            (Instruction::Xxx(slice.to_owned()))
        ) |
        241 => do_parse!(
            length: be_u24 >>
            slice: take!(length) >>
            (Instruction::Xxx(slice.to_owned()))
        ) |
        242 => do_parse!(
            length: be_u32 >>
            slice: take!(length) >>
            (Instruction::Xxx(slice.to_owned()))
        ) |
        // FontDef
        243 => do_parse!(
            number: be_u8 >>
            checksum: be_u32 >>
            scale_factor: be_u32 >>
            design_size: be_u32 >>
            directory_len: be_u8 >>
            filename_len: be_u8 >>
            directory: take!(directory_len) >>
            filename: take!(filename_len) >>
            ({
                let directory = match directory_len {
                    0 => None,
                    _ => Some(directory.to_owned()),
                };
                Instruction::FontDef(FontDef {
                    number: number as u32,
                    checksum,
                    scale_factor,
                    design_size,
                    directory,
                    filename: filename.to_owned(),
                })
            })
        ) |
        244 => do_parse!(
            number: be_u16 >>
            checksum: be_u32 >>
            scale_factor: be_u32 >>
            design_size: be_u32 >>
            directory_len: be_u8 >>
            filename_len: be_u8 >>
            directory: take!(directory_len) >>
            filename: take!(filename_len) >>
            ({
                let directory = match directory_len {
                    0 => None,
                    _ => Some(directory.to_owned()),
                };
                Instruction::FontDef(FontDef {
                    number: number as u32,
                    checksum,
                    scale_factor,
                    design_size,
                    directory,
                    filename: filename.to_owned(),
                })
            })
        ) |
        245 => do_parse!(
            number: be_u24 >>
            checksum: be_u32 >>
            scale_factor: be_u32 >>
            design_size: be_u32 >>
            directory_len: be_u8 >>
            filename_len: be_u8 >>
            directory: take!(directory_len) >>
            filename: take!(filename_len) >>
            ({
                let directory = match directory_len {
                    0 => None,
                    _ => Some(directory.to_owned()),
                };
                Instruction::FontDef(FontDef {
                    number: number as u32,
                    checksum,
                    scale_factor,
                    design_size,
                    directory,
                    filename: filename.to_owned(),
                })
            })
        ) |
        246 => do_parse!(
            number: be_u32 >>
            checksum: be_u32 >>
            scale_factor: be_u32 >>
            design_size: be_u32 >>
            directory_len: be_u8 >>
            filename_len: be_u8 >>
            directory: take!(directory_len) >>
            filename: take!(filename_len) >>
            ({
                let directory = match directory_len {
                    0 => None,
                    _ => Some(directory.to_owned()),
                };
                Instruction::FontDef(FontDef {
                    number: number as u32,
                    checksum,
                    scale_factor,
                    design_size,
                    directory,
                    filename: filename.to_owned(),
                })
            })
        ) |
        // Pre
        247 => do_parse!(
            format: be_u8 >>
            numerator: be_u32 >>
            denominator: be_u32 >>
            magnification: be_u32 >>
            comment_length: be_u8 >>
            comment: take!(comment_length) >>
            (Instruction::Pre {
                format,
                numerator,
                denominator,
                magnification,
                comment: comment.to_owned()
            })
        ) |
        248 => do_parse!(
            final_bop_pointer: be_i32 >>
            numerator: be_u32 >>
            denominator: be_u32 >>
            magnification: be_u32 >>
            tallest_height: be_i32 >>
            widest_width: be_i32 >>
            max_stack_depth: be_u16 >>
            total_no_pages: be_u16 >>
            (Instruction::Post {
                final_bop_pointer,
                numerator,
                denominator,
                magnification,
                tallest_height,
                widest_width,
                max_stack_depth,
                total_no_pages,
            })
        ) |
        249 => do_parse!(
            post_pointer: be_u32 >>
            ident: be_u8 >>
            two_two_three: parse_223 >>
            (Instruction::PostPost {
                post_pointer,
                ident,
                two_two_three,
            })
        )
    )
);
