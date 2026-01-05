package main

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"sync"
	"time"
)

func main() {
	workers := flag.Int("workers", envInt("WL_WORKERS", 8), "number of goroutines")
	holdUS := flag.Int("hold-us", envInt("WL_HOLD_US", 0), "microseconds to hold the lock")
	flag.Parse()

	if *workers <= 0 {
		fmt.Fprintln(os.Stderr, "workers must be > 0")
		os.Exit(1)
	}

	var (
		mu    sync.Mutex
		total int
		wg    sync.WaitGroup
		hold  = time.Duration(*holdUS) * time.Microsecond
	)

	wg.Add(*workers)
	for i := 0; i < *workers; i++ {
		go func() {
			defer wg.Done()
			for {
				mu.Lock()
				total++
				if hold > 0 {
					spinWait(hold)
				}
				mu.Unlock()
			}
		}()
	}
	wg.Wait()

	if total < 0 {
		fmt.Println(total)
	}
}

func envInt(name string, def int) int {
	value := os.Getenv(name)
	if value == "" {
		return def
	}
	parsed, err := strconv.Atoi(value)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid %s\n", name)
		os.Exit(1)
	}
	return parsed
}

func spinWait(d time.Duration) {
	deadline := time.Now().Add(d)
	for time.Now().Before(deadline) {
	}
}
