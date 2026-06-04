# Documentation Generation

Documentation in `git-vmr` is generated from the Git man pages using `./man-to-md.sh <man-page> <path/to/output.md>`.

For example, to generate the documentation for `git vmr commit`, run:

    docs/man-to-md.sh git-commit docs/git-vmr/commit.md

Then, update the auto-generated support matrix the newly created file.

Finally, update command entries in [git-vmr.md](git-vmr.md) and [README.md](../README.md) (if applicable) with links to the newly created file.
