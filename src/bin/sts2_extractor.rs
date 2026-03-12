use itertools::Itertools;
use rusty_spire::pck::{PckArchive, PckFileFlags, PckFileMetadata};
use rusty_spire::texture::CompressedTexture2d;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Seek};
use std::path::{Component, Path, PathBuf};

const DRY_RUN: bool = false;

fn main() -> anyhow::Result<()> {
    extract()?;
    Ok(())
}

// noinspection RsConstantConditionIf
#[allow(unused)]
fn extract() -> anyhow::Result<()> {
    let output_dir = Path::new("../sts2_data");

    let steam_dir = steamlocate::SteamDir::locate()?;
    let (sts2, library) = steam_dir
        .find_app(2868840)?
        .ok_or_else(|| anyhow::anyhow!("sts2 is not installed"))?;
    let sts2_dir = library.resolve_app_dir(&sts2);

    extract_pck(
        sts2_dir.join("SlayTheSpire2.pck"),
        output_dir.join("SlayTheSpire2.pck"),
    )?;

    let sts2_data_dir = sts2_dir.join("data_sts2_windows_x86_64");
    let sts2_dll = sts2_data_dir.join("sts2.dll");
    if !DRY_RUN {
        decompile(sts2_dll, output_dir.join("sts2.dll"))?;
    }

    Ok(())
}

// noinspection RsConstantConditionIf
#[allow(unused)]
fn extract_pck(pck_path: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let pck_path = pck_path.as_ref();
    let output_dir = output_dir.as_ref();

    if !DRY_RUN && std::fs::exists(output_dir)? {
        std::fs::remove_dir_all(output_dir)?;
    }

    let f = File::open(pck_path)?;
    let mut pck_archive = PckArchive::new_from_start(BufReader::new(f))?;

    println!("{:?}", pck_archive.metadata().header);
    let mut stats = Stats::default();

    let file_count = pck_archive.len();
    for i in 0..file_count {
        let mut f = pck_archive.by_index(i)?;
        let m = f.file_metadata();
        if i % 100 == 0 {
            println!("file: {i}/{file_count}");
        }

        stats.register(m);
        if m.flags.contains(PckFileFlags::ENCRYPTED)
            || m.flags.contains(PckFileFlags::REMOVAL)
            || m.flags.contains(PckFileFlags::DELTA)
        {
            continue;
        }

        // extract everything else
        {
            let path = PathBuf::from(&*m.path);
            if path
                .components()
                .any(|c| !matches!(c, Component::Normal(name) if name.to_str().is_some()))
            {
                anyhow::bail!("malformed path: {}", path.display());
            }

            let out_path = output_dir.join(&path);
            if !DRY_RUN {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out_file = File::create(&out_path)?;
                std::io::copy(&mut f, &mut out_file)?;
                f.rewind();
            }

            #[allow(clippy::single_match)]
            match path.extension().and_then(OsStr::to_str).unwrap_or_default() {
                "ctex" => match CompressedTexture2d::load(&mut f) {
                    Ok(texture) => {
                        if !DRY_RUN {
                            let out_file = File::create(out_path.with_added_extension("png"))?;
                            texture.images[0]
                                .write_to(BufWriter::new(out_file), image::ImageFormat::Png)?;
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {e}", path.display());
                    }
                },
                _ => {}
            }
        }
    }

    println!("stats:");
    println!("  total:     {}", stats.total);
    println!("  removals:  {}", stats.removals);
    println!("  encrypted: {}", stats.encrypted);
    println!("  delta:     {}", stats.delta);
    println!("  size == 0: {}", stats.empty);
    println!("  size <= 1: {}", stats.size_one_or_less);
    println!("extensions:");
    stats
        .extensions
        .into_iter()
        .sorted_unstable_by_key(|(ext, amount)| (Reverse(*amount), ext.to_ascii_lowercase()))
        .for_each(|(ext, amount)| println!("  `{ext}`: {amount}"));

    Ok(())
}

#[derive(Debug, Default, Clone)]
struct Stats {
    total: usize,
    encrypted: usize,
    removals: usize,
    delta: usize,
    empty: usize,
    size_one_or_less: usize,
    extensions: HashMap<String, usize>,
}

impl Stats {
    fn register(&mut self, file: &PckFileMetadata) {
        self.total += 1;
        if file.flags.contains(PckFileFlags::ENCRYPTED) {
            self.encrypted += 1;
        }
        if file.flags.contains(PckFileFlags::REMOVAL) {
            self.removals += 1;
        }
        if file.flags.contains(PckFileFlags::DELTA) {
            self.delta += 1;
        }
        if file.size == 0 {
            self.empty += 1;
        }
        if file.size <= 1 {
            self.size_one_or_less += 1;
        }

        let ext = Path::new(&*file.path)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        if let Some(count) = self.extensions.get_mut(ext) {
            *count += 1;
        } else {
            self.extensions.insert(ext.to_string(), 1);
        }
    }
}

#[derive(Debug, Default)]
pub struct DirTree {
    files: HashMap<String, PckFileMetadata>,
    dirs: HashMap<String, DirTree>,
    size: u64,
}

impl DirTree {
    pub fn add_entry(&mut self, file: PckFileMetadata) {
        let path = file.path.clone();
        let mut components: Vec<_> = path
            .split("/")
            .filter(|&s| !s.is_empty() && s != ".")
            .collect();
        if !components.is_empty() && components[0] == "res:" {
            components.remove(0);
        }

        if !components.is_empty() {
            self._add_entry_from_components(&components, file);
        }
    }

    fn _add_entry_from_components(&mut self, components: &[&str], file: PckFileMetadata) {
        self.size += file.size;
        match components {
            [] => unreachable!(),
            [name] => {
                self.files.insert(name.to_string(), file);
            }
            [dir_name, rest @ ..] => {
                let dir = self.dirs.entry(dir_name.to_string()).or_default();
                dir._add_entry_from_components(rest, file);
            }
        }
    }
}

#[allow(unused)]
fn decompile(dll_path: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let dll_path = dll_path.as_ref();
    let output_dir = output_dir.as_ref();

    if std::fs::exists(output_dir)? {
        std::fs::remove_dir_all(output_dir)?;
    }

    println!("decompiling with `ilspycmd`...");
    let status = std::process::Command::new("ilspycmd")
        .arg("--nested-directories")
        .arg("-p")
        .arg("-o")
        .arg(output_dir)
        .arg(dll_path)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("non-zero exit code: {status}");
    }
}
