function parseCount(argv) {
  const idx = argv.indexOf("--count");
  if (idx === -1 || idx + 1 >= argv.length) {
    return null;
  }
  const value = Number(argv[idx + 1]);
  return Number.isFinite(value) ? value : null;
}

function main() {
  const count = parseCount(process.argv);
  if (!count || count <= 0) {
    console.error("missing or invalid --count");
    process.exit(1);
  }

  let total = 0;
  for (let i = 0; i < count; i += 1) {
    total += i;
  }

  // Prevents the loop from being optimized away.
  if (total < 0) {
    console.log(total);
  }
}

main();
