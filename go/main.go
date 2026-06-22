// Minimal FaaS handler (Go): read all of stdin, uppercase it, write to stdout.
// Entry contract: argv/stdin in, stdout/stderr out (WASI command).
package main

import (
	"io"
	"os"
	"strings"
)

func main() {
	input, err := io.ReadAll(os.Stdin)
	if err != nil {
		os.Exit(1)
	}
	if _, err := os.Stdout.Write([]byte(strings.ToUpper(string(input)))); err != nil {
		os.Exit(1)
	}
}
