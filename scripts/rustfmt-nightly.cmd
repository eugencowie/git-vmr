@echo off
rem Windows counterpart of rustfmt-nightly.sh
mise exec rust@nightly -- rustfmt %*
