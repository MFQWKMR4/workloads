# workloads

## Cache
- URL cache: `~/.cache/wl/url/<url-hash>/source.<ext>`
- Config cache: `~/.cache/wl/config/<config-hash>/`
- When `location` is a URL, files are downloaded once and reused by URL hash, then copied into the config cache under `sources/`.
