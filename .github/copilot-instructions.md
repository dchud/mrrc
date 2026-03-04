# GitHub Copilot Instructions

## Issue Tracking with br (beads_rust)

This project uses **br (beads_rust)** for issue tracking - a Git-backed tracker designed for AI-supervised coding workflows.

**Note:** `br` is non-invasive and never executes git commands. After `br sync --flush-only`, you must manually run `git add .beads/ && git commit`.

**Key Features:**
- Dependency-aware issue tracking
- Auto-sync with Git via JSONL
- AI-optimized CLI with JSON output
- MCP server integration for Claude and other AI assistants

**CRITICAL**: Use br for ALL task tracking. Do NOT create markdown TODO lists.

### Essential Commands

```bash
# Find work
br ready --json                    # Unblocked issues
br stale --days 30 --json          # Forgotten issues

# Create and manage
br create "Title" -t bug|feature|task -p 0-4 --json
br create "Subtask" --parent <epic-id> --json  # Hierarchical subtask
br update <id> --status in_progress --json
br close <id> --reason "Done" --json

# Search
br list --status open --priority 1 --json
br show <id> --json

# Sync (CRITICAL at end of session!)
br sync --flush-only
git add .beads/
git commit -m "sync beads"
```

### Workflow

1. **Check ready work**: `br ready --json`
2. **Claim task**: `br update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** `br create "Found bug" -p 1 --deps discovered-from:<parent-id> --json`
5. **Complete**: `br close <id> --reason "Done" --json`
6. Sync and commit:
   ```bash
   br sync --flush-only
   git add .beads/
   git commit -m "sync beads"
   ```

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Git Workflow

- Always commit `.beads/issues.jsonl` with code changes
- Run `br sync --flush-only` at end of work sessions, then `git add .beads/ && git commit`

### MCP Server (Recommended)

For MCP-compatible clients (Claude Desktop, etc.), install the beads MCP server:
- Install: `pip install beads-mcp`
- Functions: `mcp__beads__ready()`, `mcp__beads__create()`, etc.

## CLI Help

Run `br <command> --help` to see all available flags for any command.
For example: `br create --help` shows `--parent`, `--deps`, `--assignee`, etc.

## Important Rules

- ✅ Use br for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Run `br sync --flush-only` at end of sessions, then git add/commit
- ✅ Run `br <cmd> --help` to discover available flags
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT commit `.beads/beads.db` (JSONL only)
