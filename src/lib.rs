//! # Dvi format
//!
//! The dvi format is an ancient format to abstract differences between different printing
//! environments. A program would be provided with the printer hardware to convert dvi into
//! instructions for the print head, which would then produce the desired content on the printed
//! page. This means that a dvi document is essentially a set of instructions for which glyphs to
//! draw where on each page.
//!
//! It has largely been superseeded, with postscript (ps) and then protable document format (pdf)
//! becoming the dominant document format.
//!
//! However, it is quite a simple protocol, and it may be useful for working with documents in this
//! format, hence the motivation for writing a library.
//!
//! A dvi file is a sequence of `Instructions`. See the [`Instruction` enum][instruction_enum] for
//! details of the different instructions contained.
//!
//! Note that currently paths must be utf8, and parsing will error if this is not true
//!
//! [instruction_enum]: ./enum.Instruction.html

//! ## Implementation notes
//!
//! An instruction is a u8, followed by 0 or more instruction-specific parameters
//! All multi-byte values are big-endian. All values are unsigned, except for 4-byte which is always
//! i32, and distance values, which are signed. Signed values use 2s-complement (same as rust).
//!
//! See SPECIFICATION.md for more details

#[macro_use]
extern crate nom;
extern crate byteorder;

mod dumper;
mod parser;
mod traits;
pub(crate) mod util;

pub use nom::IResult;
use std::io::{self, Write};

pub use traits::{Dump, Parse};

/// A font definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDef {
    /// The font number of this font (only 1 font per number + once in postamble)
    pub number: u32,
    /// Font checksum
    pub checksum: u32,
    /// How to scale the font
    pub scale_factor: u32,
    /// How to scale the font
    pub design_size: u32,
    /// Directory of the font file, default if None
    pub directory: Option<Vec<u8>>,
    /// Name of the font file
    pub filename: Vec<u8>,
}

/// A draw instruction
///
/// This is the primary unit of a dvi file. Every file is a sequence of instructions following some
/// rules, for example 'preamble only occurs once at the beginning.'
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// typeset a character and move right
    ///
    /// Typeset character number *i* from font *f* such that the reference point of the character
    /// is at (h,v). Then increase h by the width of that character. Note that a character may have
    /// zero or negative width, so one cannot be sure that h will advance after this command;
    /// but h usually does increase.
    Set(u32),
    /// typeset a rule and move right
    ///
    /// Typeset a solid black rectangle of height a and width b, with its bottom left corner
    /// at (h,v). Then set h:=h+b. If either a <= 0 or b <= 0, nothing should be typeset. Note
    /// that if b < 0, the value of h will decrease even though nothing else happens. Programs
    /// that typeset from DVI files should be careful to make the rules line up carefully with
    /// digitized characters, as explained in connection with the rule_pixels subroutine below.
    SetRule(i32, i32),
    /// typeset a character
    ///
    /// Typeset character number c from font f such that the reference point of the
    /// character is at (h,v). (The put commands are exactly like the set commands,
    /// except that they simply put out a character or a rule without moving the reference
    /// point afterwards.)
    Put(u32),
    /// typeset a rule
    ///
    /// Same as set_rule, except that h is not changed.
    PutRule(i32, i32),
    /// No-op
    ///
    /// No operation, do nothing. Any number of nop's may occur between DVI commands, but a
    /// nop cannot be inserted between a command and its parameters or between two parameters.
    Nop,
    /// beginning of page
    ///
    /// Set (h,v,w,x,y,z):=(0,0,0,0,0,0) and set the stack empty. Set the current font f to an
    /// undefined value. The ten c_i parameters can be used to identify pages. The parameter p
    /// points to the previous bop command in the file, where the first bop has p=-1.
    Bop([i32; 10], i32),
    /// ending of page
    ///
    /// End of page: Print what you have read since the previous bop. At this point the stack
    /// should be empty. (The DVI-reading programs that drive most output devices will have kept
    /// a buffer of the material that appears on the page that has just ended. This material is
    /// largely, but not entirely, in order by v coordinate and (for fixed v) by h coordinate; so
    /// it usually needs to be sorted into some order that is appropriate for the device in
    /// question. DVItype does not do such sorting.)
    Eop,
    /// save current positions
    ///
    /// Push the current values of (h,v,w,x,y,z) onto the top of the stack; do not change any
    /// of these values. Note that f is not pushed.
    Push,
    /// restore current positions
    ///
    /// Pop the top six values off of the stack and assign them to (h,v,w,x,y,z). The number of
    /// pops should never exceed the number of pushes, since it would be highly embarrassing if
    /// the stack were empty at the time of a pop command.
    Pop,
    /// Move right
    ///
    /// Set h:=h+b, i.e., move right b units. The parameter is a signed number in two's
    /// complement notation; if b < 0, the reference point actually moves left.
    Right(i32),
    /// Move right by *w*
    ///
    /// The w0 command sets h:=h+w; i.e., moves right w units. With luck, this parameterless
    /// command will usually suffice, because the same kind of motion will occur several times
    /// in succession. The other w commands set w:=b and h:=h+b. The value of b is a signed
    /// quantity in two's complement notation. This command changes the current w spacing and
    /// moves right by b.
    W(Option<i32>),
    /// Move right by *x*
    ///
    /// The parameterless x0 command sets h:=h+x; i.e., moves right x units. The x commands are
    /// like the w commands except that they involve x instead of w. The other x commands set x:=b
    /// and h:=h+b. The value of b is a signed quantity in two's complement notation. This command
    /// changes the current x spacing and moves right by b.
    X(Option<i32>),
    /// Move down
    ///
    /// Set v:=v+a, i.e., move down a units. The parameter is a signed number in two's
    /// complement notation; if a < 0, the reference point actually moves up.
    Down(i32),
    /// Move down by *y*
    ///
    /// The y0 command sets v:=v+y; i.e., moves down y units. With luck, this parameterless
    /// command will usually suffice, because the same kind of motion will occur several times
    /// in succession. The other y commands set y:=a and v:=v+a. The value of a is a signed
    /// quantity in two's complement notation. This command changes the current y spacing and
    /// moves down by a.
    Y(Option<i32>),
    /// Move down by *z*
    ///
    /// The z0 command sets v:=v+z; i.e., moves down z units. The z commands are like the y
    /// commands except that they involve z instead of y. The other z commands set z:=a and
    /// v:=v+a. The value of a is a signed quantity in two's complement notation. This command
    /// changes the current z spacing and moves down by a.
    Z(Option<i32>),
    /// Set current font
    ///
    /// Set f:=i. Font i must previously have been defined by a fnt_def instruction, as explained
    /// below. TeX82 never generates the fnt2 command, but large font numbers may prove useful for
    /// specifications of color or texture, or they may be used for special fonts that have fixed
    /// numbers in some external coding scheme.
    Font(u32),
    /// Extension to DVI primitives
    ///
    /// This command is undefined in general; it functions as a k+i+1$-byte nop unless special
    /// DVI-reading programs are being used. TeX82 generates xxx1 when a short enough \special
    /// appears, setting k to the number of bytes being sent. It is recommended that x be a string
    /// having the form of a keyword followed by possible parameters relevant to that keyword.
    Xxx(Vec<u8>),
    /// define the meaning of a font number
    ///
    /// The four-byte value c is the check sum that TeX (or whatever program generated the DVI
    /// file) found in the TFM file for this font; c should match the check sum of the font found
    /// by programs that read this DVI file.
    ///
    /// Parameter s contains a fixed-point scale factor that is applied to the character widths in
    /// font k; font dimensions in TFM files and other font files are relative to this quantity,
    /// which is always positive and less than 2^27. It is given in the same units as the other
    /// dimensions of the DVI file. Parameter d is similar to s; it is the "design size," and
    /// (like s) it is given in DVI units. Thus, font k is to be used at mag s / 1000 d times
    /// its normal size.
    ///
    /// The remaining part of a font definition gives the external name of the font, which is an
    /// ASCII string of length a+l. The number a is the length of the "area" or directory, and l
    /// is the length of the font name itself; the standard local system font area is supposed to
    /// be used when a=0. The n field contains the area in its first a bytes.
    ///
    /// Font definitions must appear before the first use of a particular font number. Once font
    /// k is defined, it must not be defined again; however, we shall see below that font
    /// definitions appear in the postamble as well as in the pages, so in this sense each font
    /// number is defined exactly twice, if at all. Like nop commands, font definitions can appear
    /// before the first bop, or between an eop and a bop.
    FontDef(FontDef),
    /// preamble
    ///
    /// The preamble contains basic information about the file as a whole and must come at the
    /// very beginning of the file. The i byte identifies DVI format; currently this byte is
    /// always set to 2. (The value i=3 is currently used for an extended format that allows a
    /// mixture of right-to-left and left-to-right typesetting. Some day we will set i=4, when
    /// DVI format makes another incompatible change - perhaps in the year 2048.)
    ///
    /// The next two parameters, num and den, are positive integers that define the units of
    /// measurement; they are the numerator and denominator of a fraction by which all dimensions
    /// in the DVI file could be multiplied in order to get lengths in units of 10^(-7) meters.
    /// (For example, there are exactly 7227 TeX points in 254 centimeters, and TeX82 works with
    /// scaled points where there are 2^16 sp in a point, so TeX82 sets num=25400000 and den=7227
    /// 2^16=473628672.
    ///
    /// The mag parameter is what TeX82 calls \mag, i.e., 1000 times the desired magnification.
    /// The actual fraction by which dimensions are multiplied is therefore m n /1000 d. Note
    /// that if a TeX source document does not call for any true dimensions, and if you change it
    /// only by specifying a different \mag setting, the DVI file that TeX creates will be
    /// completely unchanged except for the value of mag in the preamble and postamble. (Fancy
    /// DVI-reading programs allow users to override the mag setting when a DVI file is being
    /// printed.)
    ///
    /// Finally, k and x allow the DVI writer to include a comment, which is not interpreted
    /// further. The length of comment x is k, where 0 < = k < 256.
    Pre {
        format: u8,
        numerator: u32,
        denominator: u32,
        magnification: u32,
        comment: Vec<u8>,
    },
    /// postamble beginning
    ///
    /// The last page in a DVI file is followed by post; this command introduces the postamble,
    /// which summarizes important facts that TeX has accumulated about the file, making it
    /// possible to print subsets of the data with reasonable efficiency. The parameter p is a
    /// pointer to the final bop in the file. The next three parameters, num, den, and mag, are
    /// duplicates of the quantities that appeared in the preamble.
    ///
    /// Parameters l and u give respectively the height-plus-depth of the tallest page and the
    /// width of the widest page, in the same units as other dimensions of the file. These numbers
    /// might be used by a DVI-reading program to position individual "pages" on large sheets of
    /// film or paper; however, the standard convention for output on normal size paper is to
    /// position each page so that the upper left-hand corner is exactly one inch from the left
    /// and the top. Experience has shown that it is unwise to design DVI-to-printer software
    /// that attempts cleverly to center the output; a fixed position of the upper left corner
    /// is easiest for users to understand and to work with. Therefore l and u are often ignored.
    ///
    /// Parameter s is the maximum stack depth (i.e., the largest excess of push commands over
    /// pop commands) needed to process this file. Then comes t, the total number of pages (bop
    /// commands) present.
    ///
    /// The postamble continues with font definitions, which are any number of fnt_def commands
    /// as described above, possibly interspersed with nop commands. Each font number that is
    /// used in the DVI file must be defined exactly twice: Once before it is first selected by
    /// a fnt command, and once in the postamble.
    Post {
        final_bop_pointer: i32,
        numerator: u32,
        denominator: u32,
        magnification: u32,
        tallest_height: i32,
        widest_width: i32,
        max_stack_depth: u16,
        total_no_pages: u16,
    },
    /// postamble ending
    ///
    /// The last part of the postamble, following the post_post byte that signifies the end of
    /// the font definitions, contains q a pointer to the post command that started the postamble.
    /// An identification byte, i, comes next; this currently equals 2, as in the preamble.
    ///
    /// The i byte is followed by four or more bytes that are all equal to the decimal number 223
    /// (i.e., 337 in octal). TeX puts out four to seven of these trailing bytes, until the total
    /// length of the file is a multiple of four bytes, since this works out best on machines
    /// that pack four bytes per word; but any number of 223's is allowed, as long as there are
    /// at least four of them. In effect, 223 is a sort of signature that is added at the very end.
    ///
    /// This curious way to finish off a DVI file makes it feasible for DVI-reading programs to
    /// find the postamble first, on most computers, even though TeX wants to write the postamble
    /// last. Most operating systems permit random access to individual words or bytes of a file,
    /// so the DVI reader can start at the end and skip backwards over the 223's until finding
    /// the identification byte. Then it can back up four bytes, read q, and move to byte q of
    /// the file. This byte should, of course, contain the value 248 (post); now the postamble
    /// can be read, so the DVI reader discovers all the information needed for typesetting the
    /// pages. Note that it is also possible to skip through the DVI file at reasonably high
    /// speed to locate a particular page, if that proves desirable. This saves a lot of time,
    /// since DVI files used in production jobs tend to be large.
    PostPost {
        post_pointer: u32,
        ident: u8,
        two_two_three: u32,
    },
}

// See SPECIFICATION.md for opt codes
impl Instruction {
    /// Convert this instruction to a string;
    pub fn dump<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        dumper::dump(self, writer)
    }

    /// Parse an instruction from a byte slice
    pub fn parse(bytes: &[u8]) -> IResult<&[u8], Self> {
        parser::parse(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to assert that encoding and parsing is a no-op
    fn ser_de(input: Vec<Instruction>) {
        for input in input {
            let mut out = Vec::new();
            input.dump(&mut out).unwrap();
            assert_eq!(
                input,
                Instruction::parse(&out).unwrap().1,
                "serialized {:?}",
                out
            );
        }
    }

    #[test]
    fn set() {
        ser_de(vec![
            Instruction::Set(0x01),
            Instruction::Set(0x7f),
            Instruction::Set(0x7f_ff),
            Instruction::Set(0x7f_ff_ff),
            Instruction::Set(0x7f_ff_ff_ff),
        ])
    }

    #[test]
    fn set_rule() {
        ser_de(vec![
            Instruction::SetRule(10, 10),
            Instruction::SetRule(10, -10),
            Instruction::SetRule(0, -10),
            Instruction::SetRule(0, 0),
        ])
    }

    #[test]
    fn put() {
        ser_de(vec![
            Instruction::Put(0x01),
            Instruction::Put(0x7f),
            Instruction::Put(0x7f_ff),
            Instruction::Put(0x7f_ff_ff),
            Instruction::Put(0x7f_ff_ff_ff),
        ])
    }

    #[test]
    fn put_rule() {
        ser_de(vec![
            Instruction::PutRule(10, 10),
            Instruction::PutRule(10, -10),
            Instruction::PutRule(0, -10),
            Instruction::PutRule(0, 0),
        ])
    }

    #[test]
    fn nop() {
        ser_de(vec![Instruction::Nop])
    }

    #[test]
    fn bop() {
        ser_de(vec![
            Instruction::Bop([0; 10], 1),
            Instruction::Bop([100; 10], -1),
        ])
    }

    #[test]
    fn eop() {
        ser_de(vec![Instruction::Eop])
    }

    #[test]
    fn push() {
        ser_de(vec![Instruction::Push])
    }

    #[test]
    fn pop() {
        ser_de(vec![Instruction::Pop])
    }

    #[test]
    fn right() {
        ser_de(vec![
            Instruction::Right(0x00),
            Instruction::Right(0x01),
            Instruction::Right(0x7f),
            Instruction::Right(0x7f_ff),
            Instruction::Right(0x7f_ff_ff),
            Instruction::Right(0x7f_ff_ff_ff),
            Instruction::Right(0x7fu8 as i32),
            Instruction::Right(0x7f_ffu16 as i32),
            Instruction::Right(0x7f_ff_ffu32 as i32),
            Instruction::Right(0x7f_ff_ff_ffu32 as i32),
            Instruction::Right(56888),
            Instruction::Right(-8648),
        ])
    }

    #[test]
    fn w() {
        ser_de(vec![
            Instruction::W(None),
            Instruction::W(Some(0x00)),
            Instruction::W(Some(0x01)),
            Instruction::W(Some(0x7f)),
            Instruction::W(Some(0x7f_ff)),
            Instruction::W(Some(0x7f_ff_ff)),
            Instruction::W(Some(0x7f_ff_ff_ff)),
            Instruction::W(Some(-0x7f)),
            Instruction::W(Some(-0x7f_ff)),
            Instruction::W(Some(-0x7f_ff_ff)),
            Instruction::W(Some(0x7f_ff_ff_ff)),
            Instruction::W(Some(56888)),
            Instruction::W(Some(-8648)),
        ])
    }

    #[test]
    fn x() {
        ser_de(vec![
            Instruction::X(None),
            Instruction::X(Some(0x00)),
            Instruction::X(Some(0x01)),
            Instruction::X(Some(0x7f)),
            Instruction::X(Some(0x7f_ff)),
            Instruction::X(Some(0x7f_ff_ff)),
            Instruction::X(Some(0x7f_ff_ff_ff)),
            Instruction::X(Some(-0x7f)),
            Instruction::X(Some(-0x7f_ff)),
            Instruction::X(Some(-0x7f_ff_ff)),
            Instruction::X(Some(0x7f_ff_ff_ff)),
            Instruction::X(Some(56888)),
            Instruction::X(Some(-8648)),
        ])
    }

    #[test]
    fn down() {
        ser_de(vec![
            Instruction::Down(0x00),
            Instruction::Down(0x01),
            Instruction::Down(0x7f),
            Instruction::Down(0x7f_ff),
            Instruction::Down(0x7f_ff_ff),
            Instruction::Down(0x7f_ff_ff_ff),
            Instruction::Down(-0x7f),
            Instruction::Down(-0x7f_ff),
            Instruction::Down(-0x7f_ff_ff),
            Instruction::Down(0x7f_ff_ff_ff),
            Instruction::Down(56888),
            Instruction::Down(-8648),
        ])
    }

    #[test]
    fn y() {
        ser_de(vec![
            Instruction::Y(None),
            Instruction::Y(Some(0x00)),
            Instruction::Y(Some(0x01)),
            Instruction::Y(Some(0x7f)),
            Instruction::Y(Some(0x7f_ff)),
            Instruction::Y(Some(0x7f_ff_ff)),
            Instruction::Y(Some(0x7f_ff_ff_ff)),
            Instruction::Y(Some(-0x7f)),
            Instruction::Y(Some(-0x7f_ff)),
            Instruction::Y(Some(-0x7f_ff_ff)),
            Instruction::Y(Some(0x7f_ff_ff_ff)),
            Instruction::Y(Some(56888)),
            Instruction::Y(Some(-8648)),
        ])
    }

    #[test]
    fn z() {
        ser_de(vec![
            Instruction::Z(None),
            Instruction::Z(Some(0x00)),
            Instruction::Z(Some(0x01)),
            Instruction::Z(Some(0x7f)),
            Instruction::Z(Some(0x7f_ff)),
            Instruction::Z(Some(0x7f_ff_ff)),
            Instruction::Z(Some(0x7f_ff_ff_ff)),
            Instruction::Z(Some(-0x7f)),
            Instruction::Z(Some(-0x7f_ff)),
            Instruction::Z(Some(-0x7f_ff_ff)),
            Instruction::Z(Some(0x7f_ff_ff_ff)),
            Instruction::Z(Some(56888)),
            Instruction::Z(Some(-8648)),
        ])
    }

    #[test]
    fn font() {
        ser_de(vec![
            Instruction::Font(0x01),
            Instruction::Font(0x7f),
            Instruction::Font(0x7f_ff),
            Instruction::Font(0x7f_ff_ff),
            Instruction::Font(0x7f_ff_ff_ff),
        ])
    }

    #[test]
    fn xxx() {
        ser_de(vec![
            Instruction::Xxx(vec![0; 0]),
            Instruction::Xxx(vec![0; 10]),
            Instruction::Xxx(vec![0; 1000]),
            Instruction::Xxx(vec![0; 1 << 24]),
        ])
    }

    #[test]
    fn font_def() {
        ser_de(vec![
            Instruction::FontDef(FontDef {
                number: 0xfa,
                checksum: 0xdeadbeef,
                scale_factor: 0x1000,
                design_size: 0x100,
                directory: Some(Vec::from("/my/font/dir/")),
                filename: Vec::from("fontname.ext"),
            }),
            Instruction::FontDef(FontDef {
                number: 0xfafa,
                checksum: 0xdeadbeef,
                scale_factor: 0x1000,
                design_size: 0x100,
                directory: None,
                filename: Vec::from("fontname.ext"),
            }),
            Instruction::FontDef(FontDef {
                number: 0xfafafa,
                checksum: 0xdeadbeef,
                scale_factor: 0x1000,
                design_size: 0x100,
                directory: None,
                filename: Vec::from("fontname.ext"),
            }),
            Instruction::FontDef(FontDef {
                number: 0xfafafafa,
                checksum: 0xdeadbeef,
                scale_factor: 0x10000,
                design_size: 0x100000,
                directory: Some(Vec::from("/my/font/dir/")),
                filename: Vec::from("fontname.ext"),
            }),
        ])
    }
    #[test]
    fn pre() {
        ser_de(vec![Instruction::Pre {
            format: 2,
            numerator: 1000,
            denominator: 522,
            magnification: 10203,
            comment: b"Hi, I'm a comment".to_vec(),
        }])
    }

    #[test]
    fn post() {
        ser_de(vec![Instruction::Post {
            final_bop_pointer: 1023,
            numerator: 0xfa,
            denominator: 0xfa3d,
            magnification: 0xfa3df,
            tallest_height: 0xffff,
            widest_width: 0xfff,
            max_stack_depth: 0xfe24,
            total_no_pages: 0xff,
        }])
    }

    #[test]
    fn post_post() {
        ser_de(vec![
            Instruction::PostPost {
                post_pointer: 129,
                ident: 2,
                two_two_three: 3,
            },
            Instruction::PostPost {
                post_pointer: 0xffffff,
                ident: 2,
                two_two_three: 0,
            },
        ])
    }
}
