import argparse


def main() -> None:
    parser = argparse.ArgumentParser(description="Python workload loop counter")
    parser.add_argument("--count", type=int, required=True)
    args = parser.parse_args()

    count = args.count
    total = 0
    for i in range(count):
        total += i

    # Prevents the loop from being optimized away.
    if total < 0:
        print(total)


if __name__ == "__main__":
    main()
