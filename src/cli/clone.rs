use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn clone(
    working_dir: &Path,
    repository: &str,
    directory: Option<&Path>
) -> Result<()>
{
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(working_dir)
        .arg("clone")
        .arg(repository)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(directory) = directory
    {
        command.arg(directory);
    }

    let status = command.status().with_context(|| {
        let destination = directory
            .map(|directory| format!(" into '{}'", directory.display()))
            .unwrap_or_default();
        format!(
            "failed to invoke git clone '{}'{} from '{}'",
            repository,
            destination,
            working_dir.display()
        )
    })?;

    if !status.success()
    {
        bail!("git clone '{}' failed with {status}", repository);
    }

    Ok(())
}
