function parseArgs(argv) {
  const args = new Map();
  for (let i = 2; i < argv.length; i += 1) {
    const key = argv[i];
    if (key.startsWith("--") && i + 1 < argv.length) {
      args.set(key, argv[i + 1]);
      i += 1;
    }
  }
  return args;
}

function main() {
  const args = parseArgs(process.argv);
  const iterations = Number(
    args.get("--iterations") ?? process.env.WL_ITERATIONS ?? 0
  );

  if (!Number.isFinite(iterations) || iterations < 0) {
    console.error("invalid --iterations");
    process.exit(1);
  }

  let total = 0;
  if (iterations > 0) {
    for (let i = 0; i < iterations; i += 1) {
      total += i ^ (total << 1);
    }
  } else {
    for (;;) {
      total += 1;
      total ^= total << 1;
      total &= 0xffffffff;
    }
  }

  if (total === 42) {
    console.log("unexpected");
  }
}

main();
