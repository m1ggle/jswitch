# Version Scope Resolution

`jswitch` supports three Java version scopes: global, local, and session. The effective version is resolved by precedence:

```text
session > local > global
```

## Global Scope

Global scope stores the default Java version in the user config file, usually:

```text
~/.jswitch/config.toml
```

Example:

```bash
jswitch switch 17 --global
```

This sets the default Java version for directories that do not have a local or session override.

## Local Scope

Local scope stores the project Java version in the current project, usually in:

```text
.java-version
```

Example:

```bash
cd my-project
jswitch switch 11 --local
```

When running inside that project, the local version overrides the global version. This is useful when each project requires a different Java version.

## Session Scope

Session scope is temporary and applies only to the current shell session.

Example:

```bash
jswitch switch 21 --session
```

It does not permanently update the global config and does not write `.java-version`. A new shell session falls back to local or global resolution.

## Resolution Flow

When resolving the current Java version, `jswitch` checks:

1. Whether a session version is active.
2. Whether the current directory or one of its parents has a local version file.
3. Whether the global config has a default version.
4. Otherwise, no Java version is selected.

Example:

```bash
jswitch switch 17 --global
cd project-a
jswitch switch 11 --local
jswitch current       # resolves to 11
jswitch switch 21 --session
jswitch current       # resolves to 21
```

After the session override is cleared or a new shell is opened, `project-a` resolves back to `11`; outside that project, `jswitch` resolves back to the global `17`.
