package main

import (
	"flag"
	"fmt"
	"os"
	"sync"
	"time"
)

func main() {
	workers := flag.Int("workers", 8, "number of goroutines")
	holdUS := flag.Int("hold-us", 0, "microseconds to hold the lock")
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

func spinWait(d time.Duration) {
	deadline := time.Now().Add(d)
	for time.Now().Before(deadline) {
	}
}
