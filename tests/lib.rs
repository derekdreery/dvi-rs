extern crate dvi;
use dvi::Instruction;
use std::fs::File;
use std::io::Read;

fn parse(input: &[u8]) -> Vec<Instruction> {
    let mut input = input;
    let mut instructions = Vec::new();
    while input.len() > 0 {
        let instruction = match Instruction::parse(&input) {
            Result::Ok((i, inst)) => {
                input = i;
                inst
            }
            Result::Err(_) => panic!("Parse error"),
        };
        instructions.push(instruction);
    }
    instructions
}

fn dump(input: &[Instruction]) -> Vec<u8> {
    // will still reallocate but hopefully less
    let mut output = Vec::with_capacity(input.len());
    for inst in input {
        inst.dump(&mut output).unwrap();
    }
    output
}

#[test]
fn main() {
    let mut input_owned = Vec::new();
    File::open("tests/source/main.dvi")
        .unwrap()
        .read_to_end(&mut input_owned)
        .unwrap();
    let instructions = parse(&input_owned);
    // e.g.
    assert!(
        instructions[instructions.len() - 1]
            == Instruction::PostPost {
                post_pointer: 1826,
                ident: 2,
                two_two_three: 5
            }
    );

    let dumped = dump(&instructions);
    // works, but not not guaranteed to in general, since a Vec<Instruciton> has multiple valid
    // dumped representations (because u8 can be stored u32 etc)
    assert_eq!(input_owned, dumped);
    let parsed_again = parse(&dumped);

    for (i, (first, second)) in instructions.iter().zip(parsed_again.iter()).enumerate() {
        assert_eq!(
            first, second,
            "Error: {:?} != {:?}, token no {}",
            first, second, i
        );
    }
    assert_eq!(instructions, parsed_again);
    //println!("{:#?}", instructions);
    //panic!();
}
