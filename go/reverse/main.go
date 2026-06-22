// reverse: read all of stdin (UTF-8), reverse by rune, write to stdout.
// Entry contract: argv/stdin in, stdout/stderr out (WASI command).
package main

import (
	"io"
	"os"
)

func main() {
	input, err := io.ReadAll(os.Stdin)
	if err != nil {
		os.Exit(1)
	}
	runes := []rune(string(input))
	for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
		runes[i], runes[j] = runes[j], runes[i]
	}
	if _, err := os.Stdout.WriteString(string(runes)); err != nil {
		os.Exit(1)
	}
}
