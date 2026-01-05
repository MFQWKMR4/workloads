def env_int(name: str, default: int) -> int:
    value = os.getenv(name)
    if value is None:
        return default
    try:
        return int(value)
    except ValueError as exc:
        raise SystemExit(f"invalid {name}") from exc


def main() -> None:
    parser = argparse.ArgumentParser(description="Python memory staircase workload")
    parser.add_argument("--chunk-mb", type=int, default=env_int("WL_CHUNK_MB", 32))
    parser.add_argument("--steps", type=int, default=env_int("WL_STEPS", 4))
    parser.add_argument("--hold-ms", type=int, default=env_int("WL_HOLD_MS", 500))
    parser.add_argument("--release-ms", type=int, default=env_int("WL_RELEASE_MS", 500))
    args = parser.parse_args()

    if args.chunk_mb <= 0 or args.steps <= 0:
        raise SystemExit("chunk-mb and steps must be > 0")

    while True:
        chunks = []
        for _ in range(args.steps):
            chunks.append(bytearray(args.chunk_mb * 1024 * 1024))

        time.sleep(args.hold_ms / 1000.0)

        while chunks:
            chunks.pop()
            gc.collect()
            time.sleep(args.release_ms / 1000.0)


if __name__ == "__main__":
    main()
