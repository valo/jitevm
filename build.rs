use revm::primitives::{keccak256, B256};
use std::{env, fs::read_dir, path};

include!("./generator.rs");

fn write_evm_aot_cache(
    dest_file: &mut std::fs::File,
    evm_aot_cache: &[B256],
) -> std::io::Result<()> {
    writeln!(dest_file, "use std::collections::HashMap;")?;
    writeln!(dest_file, "use crate::primitives::CompiledEVM;")?;
    writeln!(dest_file, "use revm::primitives::{{Spec, B256}};")?;
    for evm_sha3 in evm_aot_cache {
        writeln!(dest_file, "use crate::evm_{};", hex::encode(evm_sha3))?;
    }

    writeln!(
        dest_file,
        "pub fn evm_cache<const CHECKED: bool, SPEC: Spec>(
    ) -> HashMap<B256, Box<dyn CompiledEVM<CHECKED, SPEC>>> {{
        HashMap::from(["
    )?;
    for evm_sha3 in evm_aot_cache {
        writeln!(
            dest_file,
            "        (
                \"{}\"
                    .parse::<B256>()
                    .unwrap(),
                Box::new(
                    evm_{}::EVMCode {{}},
                ) as Box<dyn CompiledEVM<CHECKED, SPEC>>,
            ),
    ",
            hex::encode(evm_sha3),
            hex::encode(evm_sha3)
        )?;
    }
    writeln!(
        dest_file,
        "    ])
}}"
    )?;
    Ok(())
}

fn write_lib(dest_file: &mut std::fs::File, evm_aot_cache: &[B256]) -> std::io::Result<()> {
    writeln!(dest_file, "pub mod primitives;")?;
    writeln!(dest_file, "pub mod evm_cache;")?;
    for evm_sha3 in evm_aot_cache {
        writeln!(
            dest_file,
            "mod evm_{};",
            hex::encode(evm_sha3.to_fixed_bytes())
        )?;
    }
    Ok(())
}

fn main() {
    let evm_code_dir = read_dir("./evm_code").unwrap();
    let out_dir = "evm-dynamic/src";
    let mut evm_aot_cache = Vec::new();

    for file in evm_code_dir {
        let file = file.unwrap();
        let path = file.path();

        // Read the evm code file
        let evm_bytecode = std::fs::read_to_string(&path).unwrap();
        let evm_bytecode = hex::decode(evm_bytecode).unwrap();
        println!("EVM bytecode: {}", hex::encode(&evm_bytecode));
        let evm_sha3 = keccak256(&evm_bytecode);

        let dest_path = path::Path::new(&out_dir).join(format!("evm_{}.rs", hex::encode(evm_sha3)));
        println!("cargo:rerun-if-changed={}", path.display());
        println!("cargo:rerun-if-changed={}", out_dir);
        println!("cargo:rerun-if-changed={}", dest_path.display());

        // Open the destination file in write-only mode, returns `io::Result<File>`
        let mut dest_file = std::fs::File::create(&dest_path).unwrap();

        if convert_evm_to_rust(&mut dest_file, &evm_bytecode).is_ok() {
            evm_aot_cache.push(evm_sha3);
        }
    }

    // Generate the lib.rs file, which defines all the created modules
    let dest_path = path::Path::new(&out_dir).join("lib.rs");
    let mut dest_file = std::fs::File::create(&dest_path).unwrap();
    write_lib(&mut dest_file, &evm_aot_cache).unwrap();

    println!("cargo:rerun-if-changed={}", dest_path.display());

    // Generate the hash to code mapping
    let dest_path = path::Path::new(&out_dir).join("evm_cache.rs");
    let mut dest_file = std::fs::File::create(&dest_path).unwrap();
    write_evm_aot_cache(&mut dest_file, &evm_aot_cache).unwrap();

    println!("cargo:rerun-if-changed={}", dest_path.display());
}
