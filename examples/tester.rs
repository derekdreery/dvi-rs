use dvi::{FontDef, Instruction};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result {
    let opt = Opt::from_args();
    let file = fs::read(&opt.file)?;
    Dvi::new(InstIter(&file));
    Ok(())
}

pub struct Dvi<I> {
    instr: I,
    scale: f64,
}

impl<I> Dvi<I>
where
    I: Iterator<Item = Result<Instruction, &'static str>>,
{
    fn new(mut input: I) -> Result<Self, &'static str> {
        let pre = input.next().ok_or("empty file")??;
        let scale = if let Instruction::Pre {
            format,
            numerator,
            denominator,
            magnification,
            comment,
        } = pre
        {
            if format != 2 {
                return Err("unsupported format");
            }
            eprintln!("{}", String::from_utf8_lossy(&comment));
            (numerator as f64 * magnification as f64) / (denominator as f64 * 1000.0)
        } else {
            return Err("first instruction was not a pre");
        };
        Ok(Dvi {
            instr: input,
            scale,
        })
    }
}

struct InstIter<'a>(&'a [u8]);

impl<'a> Iterator for InstIter<'a> {
    type Item = Result<Instruction, &'static str>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        let (input, inst) = match Instruction::parse(self.0) {
            Ok((i, inst)) => (i, inst),
            Err(_) => return Some(Err("malformed instruction")),
        };
        self.0 = input;
        Some(Ok(inst))
    }
}
