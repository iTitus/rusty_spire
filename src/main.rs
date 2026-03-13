use itertools::Itertools;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};
use std::path::{Component, Path, PathBuf};
use sts2_extractor::pck::{PckFile, PckHeader, PckReader};
use sts2_extractor::texture::CompressedTexture2d;

const DRY_RUN: bool = false;

// noinspection RsConstantConditionIf
fn main() -> anyhow::Result<()> {
    let dir = Path::new("C:/Program Files (x86)/Steam/steamapps/common/Slay the Spire 2");

    extract_pck(dir.join("SlayTheSpire2.pck"), "out_sts2_pck")?;
    if !DRY_RUN {
        decompile(
            dir.join("data_sts2_windows_x86_64/sts2.dll"),
            "out_sts2_dll",
        )?;
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct DirTree {
    files: HashSet<String>,
    dirs: HashMap<String, DirTree>,
    size: usize,
}

impl DirTree {
    pub fn add_entry(&mut self, file: &PckFile) {
        let mut components: Vec<_> = file.path.split("/").filter(|s| !s.is_empty()).collect();
        if !components.is_empty() && components[0].ends_with(':') {
            components.remove(0);
        }
        self._add_entry_from_components(file.size as usize, &components);
    }

    fn _add_entry_from_components(&mut self, size: usize, components: &[&str]) {
        assert!(!components.is_empty());
        self.size += size;
        match components.len() {
            0 => panic!(),
            1 => {
                self.files.insert(components[0].to_string());
            }
            _ => {
                let dir = self.dirs.entry(components[0].to_string()).or_default();
                dir._add_entry_from_components(size, &components[1..]);
            }
        }
    }
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
    let mut pck_reader = PckReader::new_from_start(BufReader::new(f))?;

    println!("{:#?}", pck_reader.pck.header);
    println!("found {} files", pck_reader.pck.files.len());
    println!(
        "found {} size<=1 files",
        pck_reader.pck.files.iter().filter(|f| f.size <= 1).count()
    );
    println!(
        "found {} removals",
        pck_reader
            .pck
            .files
            .iter()
            .filter(|f| (f.flags & PckHeader::FLAG_FILE_REMOVAL) != 0)
            .count()
    );
    println!(
        "found {} encrypted files",
        pck_reader
            .pck
            .files
            .iter()
            .filter(|f| (f.flags & PckHeader::FLAG_FILE_ENCRYPTED) != 0)
            .count()
    );
    println!(
        "found {} delta files",
        pck_reader
            .pck
            .files
            .iter()
            .filter(|f| (f.flags & PckHeader::FLAG_FILE_DELTA) != 0)
            .count()
    );

    let files: Vec<_> = pck_reader
        .pck
        .files
        .iter()
        .cloned()
        .sorted_unstable_by_key(|f| PathBuf::from(&f.path))
        .collect();
    let mut extensions: HashMap<String, usize> = HashMap::default();
    for (i, f) in files.iter().enumerate() {
        if i % 100 == 0 {
            println!("file: {i}/{}", files.len());
        }

        let ext = Path::new(&f.path)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        *extensions.entry(ext.to_string()).or_default() += 1;

        if f.size <= 1
            || (f.flags & PckHeader::FLAG_FILE_ENCRYPTED) != 0
            || (f.flags & PckHeader::FLAG_FILE_REMOVAL) != 0
            || (f.flags & PckHeader::FLAG_FILE_DELTA) != 0
        {
            continue;
        }

        // extract everything else
        {
            let bytes = pck_reader.read(&f.path)?;

            let path = Path::new(&f.path);
            if path
                .components()
                .any(|c| !matches!(c, Component::Normal(name) if name.to_str().is_some()))
            {
                anyhow::bail!("malformed path: {}", path.display());
            }

            let out_path = output_dir.join(path);
            if !DRY_RUN {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out_file = File::create(&out_path)?;
                out_file.write_all(&bytes)?;
            }

            if path.extension() == Some(OsStr::new("ctex")) {
                match CompressedTexture2d::load(&mut Cursor::new(bytes)) {
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
                }
            }
        }
    }

    println!("extensions:");
    extensions
        .into_iter()
        .sorted_unstable_by_key(|(ext, amount)| (Reverse(*amount), ext.to_ascii_lowercase()))
        .for_each(|(ext, amount)| println!("`{ext}`: {amount}"));

    Ok(())
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
