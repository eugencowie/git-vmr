# Agent Instructions

## Agent skills

### Issue tracker

Tickets live as local Markdown files in `openspec/changes/`. See `openspec/agents/issue-tracker.md`.

### Triage labels

Default canonical labels. See `openspec/agents/triage-labels.md`.

### Domain docs

Single-context layout: `CONTEXT.md` at the repo root and ADRs in `openspec/decisions/`. See `openspec/agents/domain.md`.

## Development environment

This project uses mise to manage the development environment.

- Run `mise tasks` to see the available tasks.
- Use `mise run <task>` for standard operations.
- Run `mise ls -l` to see the managed tools.
- When invoking a managed tool directly, use `mise exec -- <command> [args]` rather than invoking the tool by its bare name.
