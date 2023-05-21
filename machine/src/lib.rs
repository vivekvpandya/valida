#![feature(generic_const_exprs)]

extern crate alloc;

extern crate self as valida_machine;

use core::ops::{Index, IndexMut};
pub use p3_field::{AbstractField, Field, PrimeField, PrimeField32, PrimeField64};
pub use p3_mersenne_31::Mersenne31 as Fp;

pub mod __internal;
pub mod chip;
pub mod config;
pub mod lookup;
pub mod proof;

pub use chip::{Chip, Instruction};

pub const OPERAND_ELEMENTS: usize = 5;
pub const INSTRUCTION_ELEMENTS: usize = OPERAND_ELEMENTS + 1;
pub const CPU_MEMORY_CHANNELS: usize = 3;
pub const MEMORY_CELL_BYTES: usize = 4;
pub const LOOKUP_DEGREE_BOUND: usize = 3;

#[derive(Copy, Clone, Debug, Default)]
pub struct Word<F>(pub [F; MEMORY_CELL_BYTES]);

pub struct InstructionWord<F> {
    pub opcode: u32,
    pub operands: Operands<F>,
}

pub struct ProgramROM<F>(Vec<InstructionWord<F>>);

impl<F: PrimeField64> ProgramROM<F> {
    pub fn new(instructions: Vec<InstructionWord<F>>) -> Self {
        Self(instructions)
    }

    pub fn get_instruction(&self, pc: F) -> &InstructionWord<F> {
        &self.0[pc.as_canonical_u64() as usize]
    }
}

#[derive(Copy, Clone, Default)]
pub struct Operands<F>([F; 5]);

impl<F: Copy> Operands<F> {
    pub fn a(&self) -> F {
        self.0[0]
    }
    pub fn b(&self) -> F {
        self.0[1]
    }
    pub fn c(&self) -> F {
        self.0[2]
    }
    pub fn d(&self) -> F {
        self.0[3]
    }
    pub fn e(&self) -> F {
        self.0[4]
    }
    pub fn is_imm(&self) -> F {
        self.0[4]
    }
}

impl<F: PrimeField> Operands<F> {
    pub fn from_i32_slice(slice: &[i32]) -> Self {
        let mut operands = [F::ZERO; 5];
        for (i, &operand) in slice.iter().enumerate() {
            let abs = F::from_canonical_u32(operand.abs() as u32);
            operands[i] = if operand < 0 { -abs } else { abs };
        }
        Self(operands)
    }
}

impl From<Word<Fp>> for Fp {
    fn from(word: Word<Fp>) -> Self {
        let mut value = Fp::ZERO;
        for i in 0..MEMORY_CELL_BYTES {
            value = word.0[i] + value * Fp::from_canonical_u32(256);
        }
        value
    }
}

impl Into<[u32; MEMORY_CELL_BYTES]> for Word<Fp> {
    fn into(self) -> [u32; MEMORY_CELL_BYTES] {
        let mut result = [0; MEMORY_CELL_BYTES];
        for i in 0..MEMORY_CELL_BYTES {
            result[i] = self.0[i].as_canonical_u32();
        }
        result
    }
}

impl<F> From<[F; MEMORY_CELL_BYTES]> for Word<F> {
    fn from(bytes: [F; MEMORY_CELL_BYTES]) -> Self {
        Self(bytes)
    }
}

impl<F: Field> From<F> for Word<F> {
    fn from(bytes: F) -> Self {
        Self([F::ZERO, F::ZERO, F::ZERO, bytes])
    }
}

impl<F> Into<[F; MEMORY_CELL_BYTES]> for Word<F> {
    fn into(self) -> [F; MEMORY_CELL_BYTES] {
        self.0
    }
}

impl<T> Index<usize> for Word<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for Word<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<F> Eq for Word<F> where F: Field {}

impl<F> PartialEq for Word<F>
where
    F: Field,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }
}

impl<F> IntoIterator for Word<F> {
    type Item = F;
    type IntoIter = core::array::IntoIter<F, MEMORY_CELL_BYTES>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub trait Machine {
    type F: PrimeField64;
    fn run(&mut self, program: ProgramROM<Self::F>);
    fn prove(&self);
    fn verify();
}
