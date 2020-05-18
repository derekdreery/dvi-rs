//! functions to dump correct output to stream for each instruction type
use crate::{
    util::{byte_width, byte_width_signed},
    FontDef, Instruction,
};
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Write};

// Helper macros

/// Write out an i32 or u32, using smallest space possible
///
/// This is just purely a space-saving macro. The 5 number variant uses an Option<x32> and the 4
/// number variant uses x32
macro_rules! write_small {
    ( signed $v:expr , $writer:ident => $code1:expr , $code2:expr ,
      $code3:expr , $code4:expr ) => {{
        // help the type checker
        let out: io::Result<()> = {
            match byte_width_signed($v) {
                1 => {
                    $writer.write_u8($code1)?;
                    $writer.write_i8($v as i8)?;
                }
                2 => {
                    $writer.write_u8($code2)?;
                    $writer.write_i16::<BigEndian>($v as i16)?;
                }
                3 => {
                    $writer.write_u8($code3)?;
                    $writer.write_i24::<BigEndian>($v)?;
                }
                4 => {
                    $writer.write_u8($code4)?;
                    $writer.write_i32::<BigEndian>($v)?;
                }
                _ => { unreachable!() }
            };
            Ok(())
        };
        out
    }};

    ( signed $v:expr , $writer:ident => $code0:expr , $code1:expr ,
      $code2:expr , $code3:expr , $code4:expr ) => {{
        match $v {
            Some(v) => write_small!(signed v, $writer => $code1, $code2, $code3, $code4)?,
            None => $writer.write_u8($code0)?
        };
        Ok(())
    }};

    ( unsigned $v:expr , $writer:ident => $code1:expr ,
      $code2:expr , $code3:expr , $code4:expr ) => {{
        let out: io::Result<()> = {
            match byte_width($v) {
                1 => {
                    $writer.write_u8($code1)?;
                    $writer.write_u8($v as u8)?;
                }
                2 => {
                    $writer.write_u8($code2)?;
                    $writer.write_u16::<BigEndian>($v as u16)?;
                }
                3 => {
                    $writer.write_u8($code3)?;
                    $writer.write_u24::<BigEndian>($v)?;
                }
                4 => {
                    $writer.write_u8($code4)?;
                    $writer.write_u32::<BigEndian>($v)?;
                }
                _ => { unreachable!() }
            };
            Ok(())
        };
        out
    }};

    ( unsigned $v:expr , $writer:ident => $code0:expr , $code1:expr , $code2:expr ,
      $code3:expr , $code4:expr ) => {{
        match $v {
            Some(v) => write_small!(unsigned v, $writer => $code1, $code2, $code3, $code4),
            None => $writer.write_u8($code0)?
        };
        Ok(())
    }};
}

// Encoders

/// Dump an instruction to an impl of Write
pub(crate) fn dump<W: Write>(i: &Instruction, writer: &mut W) -> io::Result<()> {
    match *i {
        Instruction::Set(ch) => dump_set(ch, writer),
        Instruction::SetRule(a, b) => dump_set_rule(a, b, writer),
        Instruction::Put(ch) => dump_put(ch, writer),
        Instruction::PutRule(a, b) => dump_put_rule(a, b, writer),
        Instruction::Nop => writer.write_u8(138),
        Instruction::Bop(c, p) => dump_bop(c, p, writer),
        Instruction::Eop => writer.write_u8(140),
        Instruction::Push => writer.write_u8(141),
        Instruction::Pop => writer.write_u8(142),
        Instruction::Right(amt) => dump_right(amt, writer),
        Instruction::W(amt) => dump_w(amt, writer),
        Instruction::X(amt) => dump_x(amt, writer),
        Instruction::Down(amt) => dump_down(amt, writer),
        Instruction::Y(amt) => dump_y(amt, writer),
        Instruction::Z(amt) => dump_z(amt, writer),
        Instruction::Font(num) => dump_font(num, writer),
        Instruction::Xxx(ref data) => dump_xxx(&data[..], writer),
        Instruction::FontDef(ref font_def) => dump_font_def(font_def, writer),
        Instruction::Pre {
            format,
            numerator,
            denominator,
            magnification,
            ref comment,
        } => dump_pre(
            format,
            numerator,
            denominator,
            magnification,
            comment,
            writer,
        ),
        Instruction::Post {
            final_bop_pointer,
            numerator,
            denominator,
            magnification,
            tallest_height,
            widest_width,
            max_stack_depth,
            total_no_pages,
        } => dump_post(
            final_bop_pointer,
            numerator,
            denominator,
            magnification,
            tallest_height,
            widest_width,
            max_stack_depth,
            total_no_pages,
            writer,
        ),

        Instruction::PostPost {
            post_pointer,
            ident,
            two_two_three,
        } => dump_postpost(post_pointer, ident, two_two_three, writer),
    }
}

fn dump_set<W: Write>(ch: u32, writer: &mut W) -> io::Result<()> {
    match byte_width(ch) {
        1 => {
            if ch < 128 {
                writer.write_u8(ch as u8)?;
            } else {
                writer.write_u8(128)?;
                writer.write_u8(ch as u8)?;
            }
        }
        2 => {
            writer.write_u8(129)?;
            writer.write_u16::<BigEndian>(ch as u16)?;
        }
        3 => {
            writer.write_u8(130)?;
            writer.write_u24::<BigEndian>(ch)?;
        }
        4 => {
            writer.write_u8(131)?;
            writer.write_u32::<BigEndian>(ch)?;
        }
        _ => unreachable!(),
    };
    Ok(())
}

fn dump_set_rule<W: Write>(a: i32, b: i32, writer: &mut W) -> io::Result<()> {
    writer.write_u8(132)?;
    writer.write_i32::<BigEndian>(a)?;
    writer.write_i32::<BigEndian>(b)?;
    Ok(())
}

fn dump_put<W: Write>(v: u32, writer: &mut W) -> io::Result<()> {
    write_small!(unsigned v, writer => 133, 134, 135, 136)
}

fn dump_put_rule<W: Write>(a: i32, b: i32, writer: &mut W) -> io::Result<()> {
    writer.write_u8(137)?;
    writer.write_i32::<BigEndian>(a)?;
    writer.write_i32::<BigEndian>(b)?;
    Ok(())
}

fn dump_bop<W: Write>(c: [i32; 10], p: i32, writer: &mut W) -> io::Result<()> {
    writer.write_u8(139)?;
    for ci in c.iter() {
        writer.write_i32::<BigEndian>(*ci)?;
    }
    writer.write_i32::<BigEndian>(p)?;
    Ok(())
}

fn dump_right<W: Write>(v: i32, writer: &mut W) -> io::Result<()> {
    write_small!(signed v, writer => 143, 144, 145, 146)
}

fn dump_w<W: Write>(v: Option<i32>, writer: &mut W) -> io::Result<()> {
    write_small!(signed v, writer => 147, 148, 149, 150, 151)
}

fn dump_x<W: Write>(v: Option<i32>, writer: &mut W) -> io::Result<()> {
    write_small!(signed v, writer => 152, 153, 154, 155, 156)
}

fn dump_down<W: Write>(v: i32, writer: &mut W) -> io::Result<()> {
    write_small!(signed v, writer => 157, 158, 159, 160)
}

fn dump_y<W: Write>(v: Option<i32>, writer: &mut W) -> io::Result<()> {
    write_small!(signed v, writer => 161, 162, 163, 164, 165)
}

fn dump_z<W: Write>(v: Option<i32>, writer: &mut W) -> io::Result<()> {
    write_small!(signed v, writer => 166, 167, 168, 169, 170)
}

fn dump_font<W: Write>(f: u32, writer: &mut W) -> io::Result<()> {
    match byte_width(f) {
        1 => {
            if f <= 63 {
                writer.write_u8((f + 171) as u8)?;
            } else {
                writer.write_u8(235)?;
                writer.write_u8(f as u8)?;
            }
        }
        2 => {
            writer.write_u8(236)?;
            writer.write_u16::<BigEndian>(f as u16)?;
        }
        3 => {
            writer.write_u8(237)?;
            writer.write_u24::<BigEndian>(f)?;
        }
        4 => {
            writer.write_u8(238)?;
            writer.write_u32::<BigEndian>(f)?;
        }
        _ => unreachable!(),
    };
    Ok(())
}

fn dump_xxx<W: Write>(data: &[u8], writer: &mut W) -> io::Result<()> {
    assert!(
        data.len() < ::std::u32::MAX as usize,
        "The length of extention data won't fit in 32 bits"
    );
    write_small!(unsigned data.len() as u32, writer => 239, 240, 241, 242)?;
    writer.write_all(data)?;
    Ok(())
}

/// Helper for `dump_font_def`
#[inline]
fn dump_font_def_helper<W: Write>(v: u32, writer: &mut W) -> io::Result<()> {
    write_small!(unsigned v, writer => 243, 244, 245, 246)
}

fn dump_font_def<W: Write>(def: &FontDef, writer: &mut W) -> io::Result<()> {
    assert!(
        def.filename.len() <= ::std::u8::MAX as usize,
        "Filename too long in Font Definition"
    );
    assert!(
        if let Some(ref d) = def.directory {
            d.len() <= ::std::u8::MAX as usize
        } else {
            true
        },
        "Directory name too long in Font Definition"
    );
    dump_font_def_helper(def.number, writer)?;
    writer.write_u32::<BigEndian>(def.checksum)?;
    writer.write_u32::<BigEndian>(def.scale_factor)?;
    writer.write_u32::<BigEndian>(def.design_size)?;
    match def.directory {
        Some(ref d) => writer.write_u8(d.len() as u8),
        None => writer.write_u8(0),
    }?;
    writer.write_u8(def.filename.len() as u8)?;
    if let Some(ref d) = def.directory {
        writer.write_all(&d[..])?
    };
    writer.write_all(&def.filename[..])?;
    Ok(())
}

/// Pre
fn dump_pre<W: Write>(
    format: u8,
    numerator: u32,
    denominator: u32,
    magnification: u32,
    comment: &[u8],
    writer: &mut W,
) -> io::Result<()> {
    assert!(comment.len() < 0x100, "Comment length must fit into u8");
    writer.write_u8(247)?;
    writer.write_u8(format)?;
    writer.write_u32::<BigEndian>(numerator)?;
    writer.write_u32::<BigEndian>(denominator)?;
    writer.write_u32::<BigEndian>(magnification)?;
    writer.write_u8(comment.len() as u8)?;
    writer.write_all(comment)?;
    Ok(())
}

/// Post
fn dump_post<W: Write>(
    final_bop_pointer: i32,
    numerator: u32,
    denominator: u32,
    magnification: u32,
    tallest_height: i32,
    widest_width: i32,
    max_stack_depth: u16,
    total_no_pages: u16,
    writer: &mut W,
) -> io::Result<()> {
    writer.write_u8(248)?;
    writer.write_i32::<BigEndian>(final_bop_pointer)?;
    writer.write_u32::<BigEndian>(numerator)?;
    writer.write_u32::<BigEndian>(denominator)?;
    writer.write_u32::<BigEndian>(magnification)?;
    writer.write_i32::<BigEndian>(tallest_height)?;
    writer.write_i32::<BigEndian>(widest_width)?;
    writer.write_u16::<BigEndian>(max_stack_depth)?;
    writer.write_u16::<BigEndian>(total_no_pages)?;
    Ok(())
}

fn dump_postpost<W: Write>(
    post_pointer: u32,
    ident: u8,
    two_two_three: u32,
    writer: &mut W,
) -> io::Result<()> {
    writer.write_u8(249)?;
    writer.write_u32::<BigEndian>(post_pointer)?;
    writer.write_u8(ident)?;
    for _ in 0..two_two_three {
        writer.write_u8(223)?;
    }
    Ok(())
}
