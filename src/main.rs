use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Pass 1: walk the tree, grouping files by size. Symlinks and unreadable
/// entries are skipped so we never loop or double-count.
fn walk(dir: &Path, map: &mut HashMap<u64, Vec<PathBuf>>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if let Ok(md) = fs::symlink_metadata(&p) {
                if md.is_symlink() {
                    continue;
                } else if md.is_dir() {
                    walk(&p, map);
                } else if md.is_file() {
                    map.entry(md.len()).or_default().push(p);
                }
            }
        }
    }
}

/// Stream a file through a hasher in fixed chunks — constant RAM, any size.
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
    let mut map: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    walk(Path::new(&arg), &mut map);

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
        println!("\n{} copies · {} each · {} reclaimable", grp.len(), human(*sz), human(rec));
        for p in grp {
            println!("  {}", p.display());
        }
    }
    println!("\n\x1b[1;32m{} duplicate sets · {} reclaimable\x1b[0m", set.len(), human(tot));
}
