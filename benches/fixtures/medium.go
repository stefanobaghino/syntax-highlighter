// Medium-sized Go bench fixture: exercises package/import,
// struct/interface/method, generics, range, composite literals,
// closures, select, and error handling.

package main

import (
	"errors"
	"fmt"
	"io"
	"strings"
)

type Counter[T comparable] struct {
	Name      string
	values    []T
	histogram map[string]int
}

func NewCounter[T comparable](name string) *Counter[T] {
	return &Counter[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
	}
}

func (c *Counter[T]) Push(v T) *Counter[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

type Reader interface {
	Read(p []byte) (n int, err error)
}

func classify(n int) string {
	switch {
	case n < 0:
		return "negative"
	case n == 0:
		return "zero"
	case n < 10:
		return "single"
	case n < 100:
		return "double"
	default:
		return "large"
	}
}

func pipe(ch chan int, done <-chan struct{}) {
	for {
		select {
		case v, ok := <-ch:
			if !ok {
				return
			}
			fmt.Println(v)
		case <-done:
			return
		}
	}
}

var errEmpty = errors.New("empty")

func parseAll(r Reader) ([]int, error) {
	buf := make([]byte, 64)
	xs := []int{}
	for {
		n, err := r.Read(buf)
		if n > 0 {
			for _, b := range buf[:n] {
				xs = append(xs, int(b))
			}
		}
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return nil, err
		}
	}
	if len(xs) == 0 {
		return nil, errEmpty
	}
	return xs, nil
}

func main() {
	c := NewCounter[int]("ages")
	data := []int{1, 2, 2, 3, 10, 10, 10, -1}
	for _, v := range data {
		c.Push(v)
	}
	fmt.Println(c.Summary())
	for _, n := range data {
		fmt.Printf("%d -> %s\n", n, classify(n))
	}
}
