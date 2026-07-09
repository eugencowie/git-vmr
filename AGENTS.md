# Agent Instructions

## Development environment

This project uses mise to manage the development environment.

- Run `mise install` if the required tools are not installed.
- Run `mise tasks` to see the available project workflows.
- Use `mise run <task> [args]` for standard operations such as building, testing, linting, and formatting.
- When invoking a managed tool directly and no task exists, use `mise exec -- <command> [args]` rather than invoking the tool by its bare name.
