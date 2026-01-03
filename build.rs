use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let runtimes_dir = manifest_dir.join("runtimes");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("embedded_samples.rs");

    let mut files = Vec::new();
    if runtimes_dir.exists() {
        collect_files(&runtimes_dir, &mut files)?;
    }

    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    let mut output = String::new();
    output.push_str("pub struct EmbeddedSample {\n");
    output.push_str("    pub path: &'static str,\n");
    output.push_str("    pub data: &'static [u8],\n");
    output.push_str("}\n\n");
    output.push_str("pub static EMBEDDED_SAMPLES: &[EmbeddedSample] = &[\n");

    for file in &files {
        let rel = file.strip_prefix(&runtimes_dir).unwrap();
        let rel = rel.to_string_lossy().replace('\\', "/");
        let abs = file.to_string_lossy().replace('\\', "/");
        output.push_str("    EmbeddedSample {\n");
        output.push_str(&format!("        path: r#\"{}\"#,\n", rel));
        output.push_str(&format!("        data: include_bytes!(r#\"{}\"#),\n", abs));
        output.push_str("    },\n");
    }

    output.push_str("];\n");

    let mut file = fs::File::create(dest_path)?;
    file.write_all(output.as_bytes())?;
    Ok(())
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}
