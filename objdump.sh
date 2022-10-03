cargo objdump --release -- --disassemble
cargo objdump --release -- -h > dumpsect.txt
cargo objdump --release -- --disassemble-all > dumpobj.txt