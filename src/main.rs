use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Pass 1: walk the tree, grouping files by size. Symlinks and unreadable
/// entries are skipped so we never loop or double-count. `cnt` counts files
/// seen, so a long scan shows live progress instead of sitting blank.
fn walk(dir: &Path, map: &mut HashMap<u64, Vec<PathBuf>>, cnt: &mut u64) {
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if let Ok(md) = fs::symlink_metadata(&p)
                && !md.is_symlink()
            {
                if md.is_dir() {
                    walk(&p, map, cnt);
                } else if md.is_file() {
                    *cnt += 1;
                    if cnt.is_multiple_of(512) { eprint!("\r  scanned {cnt} files..."); }
                    map.entry(md.len()).or_default().push(p);
                }
            }
        }
    }
}

/// Stream a file through a hasher in fixed chunks for constant RAM on any size.
fn hash_file(p: &Path) -> io::Result<u64> {
    let mut f = File::open(p)?;
    let mut h = DefaultHasher::new();
    let mut buf = [0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        buf[..n].hash(&mut h);
    }
    Ok(h.finish())
}

/// Format a byte count as a human-readable B/KB/MB/GB/TB string.
fn human(sz: u64) -> String {
    let u = ["B", "KB", "MB", "GB", "TB"];
    let mut s = sz as f64;
    let mut i = 0;
    while s >= 1024.0 && i < 4 {
        s /= 1024.0;
        i += 1;
    }
    format!("{:.2} {}", s, u[i])
}

fn main() {
    let arg = env::args().nth(1).unwrap_or_else(|| ".".into());
    eprintln!("rust-dedupe: scanning {arg} ... (Ctrl-C to stop)");
    let mut map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut cnt = 0u64;
    walk(Path::new(&arg), &mut map, &mut cnt);
    eprintln!("\r  scanned {cnt} files, comparing same-size candidates...");

    // Pass 2: hash only files whose size collides with another's.
    let mut set: Vec<(u64, Vec<PathBuf>)> = Vec::new();
    for (sz, vec) in map {
        if vec.len() < 2 {
            continue;
        }
        let mut hm: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        for p in vec {
            if let Ok(h) = hash_file(&p) {
                hm.entry(h).or_default().push(p);
            }
        }
        for grp in hm.into_values() {
            if grp.len() > 1 {
                set.push((sz, grp));
            }
        }
    }

    // Report, biggest reclaimable win first.
    set.sort_by_key(|(sz, grp)| std::cmp::Reverse(sz * (grp.len() as u64 - 1)));
    let mut tot = 0u64;
    for (sz, grp) in &set {
        let rec = sz * (grp.len() as u64 - 1);
        tot += rec;
        println!("\n{} copies, {} each, {} reclaimable", grp.len(), human(*sz), human(rec));
        for p in grp {
            println!("  {}", p.display());
        }
    }
    let pl = if set.len() == 1 { "set" } else { "sets" };
    println!("\n\x1b[1;32m{} duplicate {pl}, {} reclaimable\x1b[0m", set.len(), human(tot));
}
