const fs = require("fs");

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
  const iterations = Number(args.get("--iterations") ?? 100);
  const size = Number(args.get("--size") ?? 4096);
  const path = args.get("--path") ?? "io_wait.dat";
  const fsync = (args.get("--fsync") ?? "true") !== "false";
  const sleepMs = Number(args.get("--sleep-ms") ?? 0);

  if (!Number.isFinite(iterations) || iterations <= 0) {
    console.error("invalid --iterations");
    process.exit(1);
  }
  if (!Number.isFinite(size) || size <= 0) {
    console.error("invalid --size");
    process.exit(1);
  }

  const fd = fs.openSync(path, "w");
  const buffer = Buffer.alloc(size);

  const sleep = (ms) => {
    if (ms <= 0) {
      return;
    }
    const shared = new Int32Array(new SharedArrayBuffer(4));
    Atomics.wait(shared, 0, 0, ms);
  };

  let i = 0;
  while (true) {
    fs.writeSync(fd, buffer, 0, buffer.length, 0);
    if (fsync) {
      fs.fsyncSync(fd);
    }
    i += 1;
    if (i >= iterations) {
      i = 0;
    }
    sleep(sleepMs);
  }

  fs.closeSync(fd);
}

main();
