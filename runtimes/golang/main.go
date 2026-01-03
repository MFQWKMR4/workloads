package main

import (
	"flag"
	"fmt"
	"os"
)

func main() {
	count := flag.Int("count", 0, "loop count")
	flag.Parse()

	if *count <= 0 {
		fmt.Fprintln(os.Stderr, "missing or invalid --count")
		os.Exit(1)
	}

	total := 0
	for i := 0; i < *count; i++ {
		total += i
	}

	if total < 0 {
		fmt.Println(total)
	}
}
