extern crate cbindgen;

use std::env;
use std::path::{Path, PathBuf};

// See:
// <https://github.com/eqrion/cbindgen/blob/master/docs.md#buildrs>
// <https://doc.rust-lang.org/cargo/reference/build-scripts.html>
// <https://doc.rust-lang.org/cargo/reference/build-script-examples.html>
fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    emit_updater_revision(&crate_dir);

    // Each header is generated from a single source file. cbindgen scans
    // exactly that file and emits the `pub extern "C"` items it defines plus
    // the C types they reference. Since each bucket file is self-contained
    // (defines its own types), there is no cross-bucket leak and no need for
    // exclusion lists in the cbindgen configs.
    generate_header(
        &crate_dir,
        "cbindgen_dart.toml",
        "src/c_api/dart.rs",
        "include/updater_dart.h",
    );
    generate_header(
        &crate_dir,
        "cbindgen_engine.toml",
        "src/c_api/engine.rs",
        "include/updater_engine.h",
    );
}

fn generate_header(crate_dir: &Path, config_name: &str, src_relative: &str, output_path: &str) {
    let config_path = crate_dir.join(config_name);
    let config = match cbindgen::Config::from_file(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("cargo:warning=Error loading {}: {e}", config_path.display());
            return;
        }
    };

    let src_path = crate_dir.join(src_relative);
    let result = cbindgen::Builder::new()
        .with_src(&src_path)
        .with_config(config)
        .generate();
    match result {
        Ok(contents) => {
            contents.write_to_file(output_path);
        }
        Err(e) => {
            println!("cargo:warning=Error generating {output_path}: {e}");
            // We don't exit non-zero here so local rust-analyzer keeps
            // working when cbindgen has an issue.
        }
    }
}

/// Stamps the updater's own git revision into the binary as `UPDATER_REVISION`.
///
/// WHY. Event eligibility for lifecycle/policy analysis must be decided by the
/// BEHAVIOUR-BEARING CLIENT CODE, not inferred from an app release version. Three
/// separate defects (server dedupe collision, whole-queue wipe, head-of-line
/// starvation) each silently zeroed the recovery numerator, and each was fixed in
/// THIS crate — so "did this client ship the fixes" is a fact about this revision.
/// Without it the server can only approximate the epoch by naming app releases,
/// which is a proxy that breaks for every other app.
///
/// Never fails the build: an unavailable git tree yields "unknown", which the
/// server treats as pre-epoch rather than trusting it.
fn emit_updater_revision(crate_dir: &Path) {
    let repo_root = crate_dir.parent().unwrap_or(crate_dir);
    let rev = std::process::Command::new("git")
        .args(["-C"])
        .arg(repo_root)
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    // Rebuild when HEAD moves, so the stamp cannot go stale silently.
    let git_head = repo_root.join(".git").join("HEAD");
    if git_head.exists() {
        println!("cargo:rerun-if-changed={}", git_head.display());
    }
    println!("cargo:rustc-env=UPDATER_REVISION={rev}");
}
