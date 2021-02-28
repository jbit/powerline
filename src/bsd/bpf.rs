#![allow(non_camel_case_types, dead_code, clippy::unusual_byte_groupings)]

use libc::{c_int, c_uint};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct bpf_insn {
    pub code: u16,
    pub jt: u8,
    pub jf: u8,
    pub k: u32,
}

const ZERO: bpf_insn = bpf_insn {
    code: 0,
    jt: 0,
    jf: 0,
    k: 0,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct bpf_stat {
    pub packets_received: c_uint,
    pub packets_dropped: c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bpf_program {
    pub bf_len: c_int,
    pub bf_insns: *const bpf_insn,
}

const LDA: u16 = 0b_000;
const LDX: u16 = 0b_001;
const STA: u16 = 0b_000;
const STX: u16 = 0b_011;
const ALU: u16 = 0b_100;
const JMP: u16 = 0b_101;
const RET: u16 = 0b_110;
const MSC: u16 = 0b_111;

#[rustfmt::skip]
mod ld {
    pub const W: u16 = 0b_00_000;
    pub const H: u16 = 0b_01_000;
    pub const B: u16 = 0b_10_000;

    pub const IMM: u16 = 0b_000_00_000;
    pub const ABS: u16 = 0b_001_00_000;
    pub const IND: u16 = 0b_010_00_000;
    pub const MEM: u16 = 0b_011_00_000;
    pub const LEN: u16 = 0b_100_00_000;
    pub const MSH: u16 = 0b_101_00_000;
}

#[rustfmt::skip]
mod alu {
    pub const K: u16 = 0b_0_000;
    pub const X: u16 = 0b_1_000;

    pub const ADD: u16 = 0b_0000_0_000;
    pub const SUB: u16 = 0b_0001_0_000;
    pub const MUL: u16 = 0b_0010_0_000;
    pub const DIV: u16 = 0b_0011_0_000;
    pub const OR:  u16 = 0b_0100_0_000;
    pub const AND: u16 = 0b_0101_0_000;
    pub const LSH: u16 = 0b_0110_0_000;
    pub const RSH: u16 = 0b_0111_0_000;
    pub const NEG: u16 = 0b_1000_0_000;
}

#[rustfmt::skip]
mod jmp {
    pub const K: u16 = 0b_0_000;
    pub const X: u16 = 0b_1_000;

    pub const JA:   u16 = 0b_0000_0_000;
    pub const JEQ:  u16 = 0b_0001_0_000;
    pub const JGT:  u16 = 0b_0010_0_000;
    pub const JGE:  u16 = 0b_0011_0_000;
    pub const JSET: u16 = 0b_0100_0_000;
}

#[rustfmt::skip]
mod ret {
    pub const K: u16 = 0b_00_000;
    pub const X: u16 = 0b_01_000;
    pub const A: u16 = 0b_10_000;
}

#[rustfmt::skip]
mod msc {
    pub const TAX: u16 = 0b_00000_000;
    pub const TXA: u16 = 0b_00001_000;
}

#[rustfmt::skip]
pub mod instructions {
    use super::*;
    pub const fn ldaw_abs(pktaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::W | ld::ABS, k: pktaddr, ..ZERO } }
    pub const fn ldah_abs(pktaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::H | ld::ABS, k: pktaddr, ..ZERO } }
    pub const fn ldab_abs(pktaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::B | ld::ABS, k: pktaddr, ..ZERO } }
    pub const fn ldaw_ind(pktaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::W | ld::IND, k: pktaddr, ..ZERO } }
    pub const fn ldah_ind(pktaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::H | ld::IND, k: pktaddr, ..ZERO } }
    pub const fn ldab_ind(pktaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::B | ld::IND, k: pktaddr, ..ZERO } }
    pub const fn lda_len ()                     -> bpf_insn { bpf_insn { code: LDA | ld::W | ld::LEN, ..ZERO } }
    pub const fn lda_imm (k: u32)               -> bpf_insn { bpf_insn { code: LDA | ld::W | ld::IMM, k, ..ZERO } }
    pub const fn lda_mem (memaddr: u32)         -> bpf_insn { bpf_insn { code: LDA | ld::W | ld::MEM, k: memaddr, ..ZERO } }

    pub const fn ldx_len()                      -> bpf_insn { bpf_insn { code: LDX | ld::W | ld::LEN, ..ZERO } }
    pub const fn ldx_imm(imm: u32)              -> bpf_insn { bpf_insn { code: LDX | ld::W | ld::IMM, k: imm, ..ZERO } }
    pub const fn ldx_mem(memaddr: u32)          -> bpf_insn { bpf_insn { code: LDX | ld::W | ld::MEM, k: memaddr, ..ZERO } }
    pub const fn ldx_msh(pktaddr: u32)          -> bpf_insn { bpf_insn { code: LDX | ld::B | ld::MSH, k: pktaddr, ..ZERO } }

    pub const fn sta_mem(memaddr: u32)          -> bpf_insn { bpf_insn { code: STA, k: memaddr, ..ZERO } }

    pub const fn stx_mem(memaddr: u32)          -> bpf_insn { bpf_insn { code: STX, k: memaddr, ..ZERO } }

    pub const fn add(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::ADD | alu::K, k, ..ZERO } }
    pub const fn sub(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::SUB | alu::K, k, ..ZERO } }
    pub const fn mul(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::MUL | alu::K, k, ..ZERO } }
    pub const fn div(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::DIV | alu::K, k, ..ZERO } }
    pub const fn or (k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::OR  | alu::K, k, ..ZERO } }
    pub const fn and(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::AND | alu::K, k, ..ZERO } }
    pub const fn lsh(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::LSH | alu::K, k, ..ZERO } }
    pub const fn rsh(k: u32)                    -> bpf_insn { bpf_insn { code: ALU | alu::RSH | alu::K, k, ..ZERO } }
    pub const fn addx()                         -> bpf_insn { bpf_insn { code: ALU | alu::ADD | alu::X, ..ZERO } }
    pub const fn subx()                         -> bpf_insn { bpf_insn { code: ALU | alu::SUB | alu::X, ..ZERO } }
    pub const fn mulx()                         -> bpf_insn { bpf_insn { code: ALU | alu::MUL | alu::X, ..ZERO } }
    pub const fn divx()                         -> bpf_insn { bpf_insn { code: ALU | alu::DIV | alu::X, ..ZERO } }
    pub const fn orx ()                         -> bpf_insn { bpf_insn { code: ALU | alu::OR  | alu::X, ..ZERO } }
    pub const fn andx()                         -> bpf_insn { bpf_insn { code: ALU | alu::AND | alu::X, ..ZERO } }
    pub const fn lshx()                         -> bpf_insn { bpf_insn { code: ALU | alu::LSH | alu::X, ..ZERO } }
    pub const fn rshx()                         -> bpf_insn { bpf_insn { code: ALU | alu::RSH | alu::X, ..ZERO } }
    pub const fn neg()                          -> bpf_insn { bpf_insn { code: ALU | alu::NEG, ..ZERO } }

    pub const fn jmp  (k: u32)                  -> bpf_insn { bpf_insn { code: JMP | jmp::JA   | jmp::K, k, ..ZERO } }
    pub const fn jeq  (jt: u8, jf: u8, k: u32)  -> bpf_insn { bpf_insn { code: JMP | jmp::JEQ  | jmp::K, jt, jf, k } }
    pub const fn jgt  (jt: u8, jf: u8, k: u32)  -> bpf_insn { bpf_insn { code: JMP | jmp::JGT  | jmp::K, jt, jf, k } }
    pub const fn jge  (jt: u8, jf: u8, k: u32)  -> bpf_insn { bpf_insn { code: JMP | jmp::JGE  | jmp::K, jt, jf, k } }
    pub const fn jset (jt: u8, jf: u8, k: u32)  -> bpf_insn { bpf_insn { code: JMP | jmp::JSET | jmp::K, jt, jf, k } }
    pub const fn jeqx (jt: u8, jf: u8)          -> bpf_insn { bpf_insn { code: JMP | jmp::JEQ  | jmp::X, jt, jf, ..ZERO } }
    pub const fn jgtx (jt: u8, jf: u8)          -> bpf_insn { bpf_insn { code: JMP | jmp::JGT  | jmp::X, jt, jf, ..ZERO } }
    pub const fn jgex (jt: u8, jf: u8)          -> bpf_insn { bpf_insn { code: JMP | jmp::JGE  | jmp::X, jt, jf, ..ZERO } }
    pub const fn jsetx(jt: u8, jf: u8)          -> bpf_insn { bpf_insn { code: JMP | jmp::JSET | jmp::X, jt, jf, ..ZERO } }

    pub const fn ret (k: u32)                   -> bpf_insn { bpf_insn { code: RET | ret::K, k, ..ZERO } }
    pub const fn reta()                         -> bpf_insn { bpf_insn { code: RET | ret::A, ..ZERO } }

    pub const fn tax()                          -> bpf_insn { bpf_insn { code: MSC | msc::TAX, ..ZERO } }
    pub const fn txa()                          -> bpf_insn { bpf_insn { code: MSC | msc::TXA, ..ZERO } }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ethertype_filter() {
        #[rustfmt::skip]
        let expected = [
            bpf_insn { code: 0x28, jt: 0, jf: 0, k: 12 },
            bpf_insn { code: 0x15, jt: 0, jf: 1, k: 0x1337 },
            bpf_insn { code: 0x06, jt: 0, jf: 0, k: 0x40000 },
            bpf_insn { code: 0x06, jt: 0, jf: 0, k: 0x00000 },
        ];
        use instructions::*;
        assert_eq!(expected[0], ldah_abs(12));
        assert_eq!(expected[1], jeq(0, 1, 0x1337));
        assert_eq!(expected[2], ret(0x40000));
        assert_eq!(expected[3], ret(0x00000));
    }
}
