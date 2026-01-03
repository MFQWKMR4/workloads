use crate::orchestrator::config::ConfigError;
use std::error::Error;
use std::fs;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embedded_samples.rs"));

pub(crate) fn write_samples(output_dir: &Path) -> Result<(), Box<dyn Error>> {
    if output_dir.exists() {
        return Err(Box::new(ConfigError(format!(
            "output directory '{}' already exists",
            output_dir.display()
        ))));
    }

    fs::create_dir_all(output_dir)?;
    for sample in EMBEDDED_SAMPLES {
        let dest = output_dir.join(sample.path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, sample.data)?;
    }

    println!(
        "samples: wrote {} files to {}",
        EMBEDDED_SAMPLES.len(),
        output_dir.display()
    );

    Ok(())
}
