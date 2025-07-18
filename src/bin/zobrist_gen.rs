use std::{fs::File, io::{BufWriter, Write}};
use rand::Rng;

const PIECE_TYPES: usize = 6;
const COLORS: usize = 2;
const SQUARES: usize = 64;
const CASTLING_RIGHTS: usize = 16;  // 2^4 = 16, encoding each set independently
const EN_PASSANT_FILES: usize = 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();
    let file = File::create("src/constants/zobrist.rs").expect("Failed to create file");
    let mut writer = BufWriter::new(file);

    writeln!(writer, "// Generated by `src/bin/zobrist_gen.rs`\n")?;

    writeln!(writer, "// Zobrist keys for [piece][color][square]")?;
    writeln!(writer, "pub const ZOBRIST_PIECE: [[[u64; {SQUARES}]; {COLORS}]; {PIECE_TYPES}] = [")?;
    for _ in 0..PIECE_TYPES {
        writeln!(writer, "\t[")?;
        for _ in 0..COLORS {
            writeln!(writer, "\t\t[")?;
            for _ in 0..SQUARES {
                writeln!(writer, "\t\t\t0x{:016x},", rng.random::<u64>())?;
            }
            writeln!(writer, "\t\t],")?;
        }
        writeln!(writer, "\t],")?;
    }
    writeln!(writer, "];\n")?;

    writeln!(writer, "pub const ZOBRIST_CASTLING: [u64; {CASTLING_RIGHTS}] = [")?;
    for _ in 0..CASTLING_RIGHTS {
        writeln!(writer, "    0x{:016x},", rng.random::<u64>())?;
    }
    writeln!(writer, "];\n")?;

    writeln!(writer, "pub const ZOBRIST_EN_PASSANT_FILE: [u64; {EN_PASSANT_FILES}] = [")?;
    for _ in 0..EN_PASSANT_FILES {
        writeln!(writer, "    0x{:016x},", rng.random::<u64>())?;
    }
    writeln!(writer, "];\n")?;

    writeln!(writer, "pub const ZOBRIST_SIDE_BLACK: u64 = 0x{:016x};", rng.random::<u64>())?;

    Ok(())
}
