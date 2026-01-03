# workloads

## Cache
- URL cache: `~/.cache/wl/url/<url-hash>/source.<ext>`
- Config cache: `~/.cache/wl/config/<config-hash>/`
- When `location` is a URL, files are downloaded once and reused by URL hash, then copied into the config cache under `sources/`.

## Config
- Use `location` to point to a URL or local file path; if omitted, the default runtime sample under `./runtimes/<lang>/main.xx` is used.

## perf usage (cheat sheet)
Goal: check CPU busy or idle
- `sudo perf record -a -e cpu-clock -- sleep 10`
- `sudo perf report`
- If `cpuidle_idle_call` dominates, CPU is mostly idle.

Goal: find hot user-space functions
- `sudo perf record -a -F 999 -e cpu-clock -- sleep 10`
- `sudo perf report --pid <pid1,pid2,...>`
- Enter the `node`/`go`/`python` line, then `Annotate`.

Goal: check kernel-heavy behavior (syscalls, scheduler)
- `sudo perf record -a -F 999 -e cpu-clock -- sleep 10`
- `sudo perf report --pid <pid1,pid2,...>`, then `Zoom into Kernel DSO`
- Look for `schedule`, `futex`, `__wake_up`, `sys_*`.

Goal: diagnose lock contention (futex)
- `sudo perf lock record -a -- sleep 10`
- `sudo perf lock report`

Goal: diagnose scheduling latency
- `sudo perf sched record -a -- sleep 10`
- `sudo perf sched latency`

## Perf
- よく使う例:
  - CPUサンプリング: `perf record -F 99 -p <pid> -- sleep 30` / `perf report`
  - 関数別のホットパス: `perf record -g -p <pid> -- sleep 30` / `perf report --stdio`
  - システム全体: `sudo perf record -a -- sleep 30`
  - コールグラフ(折りたたみ): `perf script | ./stackcollapse-perf.pl > out.folded`
  - トレース: `perf trace -p <pid>`
  - 統計: `perf stat -p <pid> -- sleep 10`
- よく使うオプション:
  - `-p <pid>`: 対象プロセスを指定
  - `-a`: システム全体を対象
  - `-g`: コールグラフを収集
  - `-F <Hz>`: サンプリング周波数
  - `-- sleep N`: 計測時間をN秒に制限
  - `--stdio`: `perf report`を標準出力に表示
