fn main() {
    // Live wallpapers are rendered by spawning `mpv.exe` as a child process,
    // so the build only needs to make `mpv.exe` available next to the built
    // binaries. There is no libmpv linking (the app spawns mpv rather than
    // embedding it), so no import library is required.
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let repo_root = std::path::Path::new(&crate_dir)
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&crate_dir));

    println!(
        "cargo:rerun-if-changed={}",
        repo_root.join("mpv").join("mpv.exe").display()
    );

    // Copy into the default target profile directories.
    for profile in ["debug", "release"] {
        copy_mpv_exe(&repo_root, &repo_root.join("target").join(profile));
    }

    // Also cover custom target directories via the OUT_DIR ancestry
    // (…/target/<profile>/build/<crate>/out -> …/target/<profile>).
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        if let Some(target_dir) = std::path::Path::new(&out_dir).ancestors().nth(3) {
            copy_mpv_exe(&repo_root, target_dir);
        }
    }
}

/// Copies `mpv/mpv.exe` into `target_dir` only when it is missing or its size
/// differs, so repeated builds do not re-copy a ~114 MB binary.
fn copy_mpv_exe(repo_root: &std::path::Path, target_dir: &std::path::Path) {
    let src = repo_root.join("mpv").join("mpv.exe");
    if !src.exists() || !target_dir.exists() {
        return;
    }
    let dest = target_dir.join("mpv.exe");

    let needs_copy = match (std::fs::metadata(&src), std::fs::metadata(&dest)) {
        (Ok(src_meta), Ok(dest_meta)) => src_meta.len() != dest_meta.len(),
        (Ok(_), Err(_)) => true,
        _ => false,
    };

    if needs_copy {
        let _ = std::fs::copy(&src, &dest);
    }
}
