# workloads

## Usage

- to generate workloads
```bash
workloads gen -c ./runconfig/cpu.yaml
```

- You can define workloads to generate by runconfig yaml file.
```yaml
steps:
  - id: python-sample
    runtime: python
    parallel:
      processes: 2
    duration_ms: 10000
    env:
      WL_CHUNK_MB: "32"
      WL_STEPS: "4"
      WL_HOLD_MS: "500"
      WL_RELEASE_MS: "500"
  - id: nodejs-cpu
    runtime: node
    location: runtimes/node/cpu.js
    duration_ms: 30000
    env:
      WL_ITERATIONS: "0"
  - id: go-sample
    runtime: golang
    parallel:
      processes: 2
    duration_ms: 10000
    env:
      WL_WORKERS: "8"
      WL_HOLD_US: "50"
  - id: nodejs-remote
    runtime: node
    location: https://gist.githubusercontent.com/MFQWKMR4/7af188c060f3d24128fbd06a123c63f3/raw/c946731695267c9f9d186be38c445c54d400b41b/test.js
    duration_ms: 10000
    env:
      WL_ITERATIONS: "100"
      WL_SIZE: "4096"
      WL_FSYNC: "true"
```

## Cache
- Base dir: `./tmp_workspace/`
- URL cache: `./tmp_workspace/url/<url-hash>/source.<ext>`
- Config cache: `./tmp_workspace/config/<config-hash>/`
- When `location` is a URL, files are downloaded once and reused by URL hash, then copied into the config cache under `sources/`.

## Config
- Use `location` to point to a URL or local file path; if omitted, the default runtime sample under `./runtimes/<lang>/main.xx` is used.
- Prefer `env` for runtime parameters; `args` is supported as a fallback.
- Use `wrapper` to prefix execution (e.g. `strace -f -c`).
