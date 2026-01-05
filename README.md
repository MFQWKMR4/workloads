# workloads

Workloads is a CLI for running repeatable CPU/IO/memory workloads across
multiple runtimes. It supports local or remote workload sources, optional
wrappers for perf/strace, and a workspace cache for reproducible runs.

## Features
- Run workloads for node/python/go/shell/bin runtimes
- Local or remote source files via `location`
- Per-step environment variables and optional args
- Wrapper commands to attach profilers and tracers
- Parallel steps by default; dependency-based flow control available
- Workspace cache under `./tmp_workspace`

## Installation
```bash
cargo install --git https://github.com/MFQWKMR4/workloads
```

## Quick start
```bash
workloads gen -c ./runconfig/cpu.yaml
```

## Configuration
`runconfig` is YAML with `steps`:
```yaml
steps:
  - id: nodejs-cpu
    runtime: node
    location: runtimes/node/cpu.js
    duration_ms: 30000
    env:
      WL_ITERATIONS: "0"
```

### Fields
- `id` (string, optional): step identifier
- `runtime` (string, required): `node`, `python`, `golang`, `shell`, `bin`
- `location` (string, optional): URL or local path to a source file
- `env` (object, optional): environment variables (preferred)
- `args` (array, optional): command args (fallback)
- `duration_ms` (number, optional): stop processes after this time
- `stdout` (bool, optional): stream stdout for each process
- `wrapper` (string, optional): prefix command (e.g. `strace -f -c`)
- `parallel.processes` (number, optional): number of processes
- `depends_on` (array, optional): dependency rules

## Examples
Local file:
```yaml
steps:
  - id: nodejs-cpu
    runtime: node
    location: runtimes/node/cpu.js
    duration_ms: 30000
    env:
      WL_ITERATIONS: "0"
```

Remote file:
```yaml
steps:
  - id: nodejs-remote
    runtime: node
    location: https://example.com/workloads/io_wait.js
    duration_ms: 10000
    env:
      WL_ITERATIONS: "100"
      WL_SIZE: "4096"
      WL_FSYNC: "true"
```

Wrapper command:
```yaml
steps:
  - id: go-contention
    runtime: golang
    location: runtimes/golang/main.go
    wrapper: "strace -f -c"
    env:
      GOMAXPROCS: "16"
      WORKERS: "64"
      HOLD_MS: "50"
```

Parallel processes:
```yaml
steps:
  - id: python-mem
    runtime: python
    parallel:
      processes: 2
    duration_ms: 10000
    env:
      WL_CHUNK_MB: "32"
      WL_STEPS: "4"
```

Dependency example:
```yaml
steps:
  - id: go-contention
    runtime: golang
    location: runtimes/golang/main.go

  - id: trace
    runtime: shell
    depends_on:
      - id: go-contention
        when: started
    command: 'p"strace -f -p {go-contention:pid}"'
```

## Placeholder expansion
- Use `p"..."` to enable expansion in `command`, `wrapper`, and `env` values.
- `{step_id:pid}` expands to the first PID.
- `{step_id:pid,}` expands to comma-separated PIDs.

## Cache and workspace
- Base dir: `./tmp_workspace/`
- URL cache: `./tmp_workspace/url/<url-hash>/source.<ext>`
- Source cache: `./tmp_workspace/source/<location-hash>/`
