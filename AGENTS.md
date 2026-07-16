# Agent Instructions

## Development environment

This project uses mise to manage the development environment.

- Run `mise install` if the required tools are not installed.
- Run `mise tasks` to see the available project workflows.
- Use `mise run <task> [args]` for standard operations such as building, testing, linting, and formatting.
- When invoking a managed tool directly and no task exists, use `mise exec -- <command> [args]` rather than invoking the tool by its bare name.

## Agent skills

### Issue tracker

Tickets live as local Markdown files in `openspec/specs/`. See `openspec/agents/issue-tracker.md`.

### Triage labels

Default canonical labels. See `openspec/agents/triage-labels.md`.

### Domain docs

Single-context layout: `CONTEXT.md` at the repo root and ADRs in `openspec/decisions/`. See `openspec/agents/domain.md`.
