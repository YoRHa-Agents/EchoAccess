# Sync Workflow

## How Sync Works

EchoAccess uses a three-state model for synchronization:

1. **Source State**: The intended configuration (from profile + cloud storage)
2. **Target State**: The rendered result after applying transforms and overrides
3. **Actual State**: What currently exists on disk

The sync engine compares these states and generates a plan:

```
Source (cloud) → Transform → Target (expected) → Diff → Actual (disk)
                                                   ↓
                                              Approval Queue
```

## Conflict Resolution

When both local and cloud copies have changed, EchoAccess performs a 3-way merge:

- **Clean merge**: Changes don't overlap — applied automatically
- **Conflict**: Changes overlap — queued for user approval

## Approval Queue

Local modifications are not automatically uploaded. They enter an approval queue:

```bash
echoax-cli sync check     # See pending changes
echoax-cli sync upload    # Upload approved changes
echoax-cli sync download  # Download from cloud
```

## Triggers

Sync can be triggered three ways:

1. **File watcher**: Automatically detects config file changes
2. **Scheduler**: Periodic sync at configurable intervals
3. **Manual**: Via CLI commands or TUI actions
