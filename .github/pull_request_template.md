## What and why

One change per pull request: what it changes and why. Link the issue if there is one.

## Checks

- [ ] `bun run format:fix`
- [ ] `bun run lint:fix`
- [ ] `bun run typecheck`
- [ ] `bun run test`
- [ ] `bun run test:e2e` (if the change touches the app's windows)
- [ ] Generated files written again with their commands, not by hand
- [ ] AGENTS.md or the doc in `agent-docs/` updated if a rule or a place it names changed
