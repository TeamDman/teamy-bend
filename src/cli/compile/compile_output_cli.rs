// SPDX-License-Identifier: MPL-2.0
use eyre::Context;
use eyre::Result;
use eyre::eyre;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileId(u64, u64);

#[cfg(windows)]
fn file_id(path: &Path) -> Result<FileId> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION;
    use windows::Win32::Storage::FileSystem::GetFileInformationByHandle;
    let file = fs::File::open(path).wrap_err("cannot open input/output identity")?;
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: File owns the live handle throughout this query and the output is writable.
    unsafe { GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &raw mut information) }
        .wrap_err("cannot query input/output identity")?;
    Ok(FileId(
        u64::from(information.dwVolumeSerialNumber),
        u64::from(information.nFileIndexHigh) << 32 | u64::from(information.nFileIndexLow),
    ))
}

#[cfg(unix)]
fn file_id(path: &Path) -> Result<FileId> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(path).wrap_err("cannot query input/output identity")?;
    Ok(FileId(metadata.dev(), metadata.ino()))
}

/// Validated output names, with identities of every loaded source file.
#[derive(Debug)]
pub(super) struct OutputPlan {
    pub output: PathBuf,
    pub cache: Option<PathBuf>,
    force: bool,
    inputs: Vec<(PathBuf, FileId)>,
}

impl OutputPlan {
    pub fn new(output: &Path, force: bool, inputs: &[PathBuf], gpu: bool) -> Result<Self> {
        let name = output
            .file_name()
            .ok_or_else(|| eyre!("compiled output needs a file name"))?;
        let parent = output
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let output = fs::canonicalize(parent)
            .wrap_err("cannot resolve output directory")?
            .join(name);
        let cache = gpu.then(|| {
            let mut name = output.as_os_str().to_owned();
            name.push(".gpu");
            PathBuf::from(name)
        });
        let inputs = inputs
            .iter()
            .map(|path| {
                let canonical = fs::canonicalize(path).wrap_err("cannot resolve compiled input")?;
                let identity = file_id(&canonical)?;
                Ok((canonical, identity))
            })
            .collect::<Result<Vec<_>>>()?;
        let plan = Self {
            output,
            cache,
            force,
            inputs,
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_file(&self.output)?;
        if let Some(cache) = &self.cache {
            self.validate_file(cache)?;
        }
        Ok(())
    }

    fn validate_file(&self, output: &Path) -> Result<()> {
        match fs::symlink_metadata(output) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err(eyre!(
                        "compiled output must be a regular file, not a directory or symlink"
                    ));
                }
                let canonical =
                    fs::canonicalize(output).wrap_err("cannot resolve existing output")?;
                let identity = file_id(output)?;
                if self
                    .inputs
                    .iter()
                    .any(|(path, id)| *path == canonical || *id == identity)
                {
                    return Err(eyre!(
                        "compiled output cannot replace a source input or its hard link"
                    ));
                }
                if !self.force {
                    return Err(eyre!(
                        "cannot create compiled output (use --force to replace an existing file)"
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).wrap_err("cannot inspect compiled output"),
        }
        Ok(())
    }

    pub fn install(&self, staged: &Path, destination: &Path) -> Result<()> {
        self.validate_file(destination)?;
        if self.force {
            fs::rename(staged, destination).wrap_err("cannot replace compiled output")
        } else {
            // Both names are on the output filesystem. A hard link publishes
            // completed bytes atomically and cannot replace a concurrently created file.
            fs::hard_link(staged, destination).wrap_err("cannot create compiled output")
        }
    }
}

/// Only this invocation's newly created staging directory is removed on drop.
#[derive(Debug)]
pub(super) struct Stage(pub PathBuf);

impl Stage {
    pub fn new(output: &Path) -> Result<Self> {
        let parent = output
            .parent()
            .ok_or_else(|| eyre!("output directory missing"))?;
        for _ in 0..128 {
            let name = format!(
                ".teamy-bend-build-{}-{}",
                std::process::id(),
                NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
            );
            let path = parent.join(name);
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(error).wrap_err("cannot create compilation staging directory");
                }
            }
        }
        Err(eyre!("cannot reserve a compilation staging directory"))
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}
