use crate::render::Rendered;

/// One child repo's captured command output, ready for rendering.
pub struct ChildOutput<'a>
{
    pub repo: &'a str,
    pub stdout: &'a [u8],
    pub stderr: &'a [u8]
}

/// Renders foreach results: per-repo chrome and child stdout in repo order,
/// then every child's stderr replayed in repo order.
pub fn foreach(children: &[ChildOutput], quiet: bool) -> Rendered
{
    let mut stdout = String::new();
    let mut stderr = String::new();

    for child in children
    {
        if !quiet
        {
            stdout.push_str(&format!("Entering '{}'\n", child.repo));
        }
        stdout.push_str(&String::from_utf8_lossy(child.stdout));
    }

    for child in children
    {
        stderr.push_str(&String::from_utf8_lossy(child.stderr));
    }

    Rendered { stdout, stderr }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn renders_chrome_and_stdout_in_repo_order_then_stderr()
    {
        // Arrange
        let children = vec![
            ChildOutput {
                repo: "backend",
                stdout: b"built\n",
                stderr: b"warning: slow\n"
            },
            ChildOutput { repo: "frontend", stdout: b"ok\n", stderr: b"" },
        ];

        // Act
        let rendered = foreach(&children, false);

        // Assert
        assert_eq!(
            rendered.stdout,
            "Entering 'backend'\nbuilt\nEntering 'frontend'\nok\n"
        );
        assert_eq!(rendered.stderr, "warning: slow\n");
    }

    #[test]
    fn quiet_suppresses_chrome_but_not_child_output()
    {
        // Arrange
        let children = vec![ChildOutput {
            repo: "backend",
            stdout: b"built\n",
            stderr: b""
        }];

        // Act
        let rendered = foreach(&children, true);

        // Assert
        assert_eq!(rendered.stdout, "built\n");
    }
}
