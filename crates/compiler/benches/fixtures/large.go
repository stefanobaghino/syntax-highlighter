// Generated Go bench fixture. Not hand-maintained —
// regenerate via benches/fixtures/gen_go.py if shapes need tweaking.

package main

import (
	"errors"
	"fmt"
	"io"
	"strings"
)

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


type Counter0[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter0[T comparable](name string, tag T) *Counter0[T] {
	return &Counter0[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter0[T]) Push(v T) *Counter0[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter0[T]) Size() int { return len(c.values) }

func (c *Counter0[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter0[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe0(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter1[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter1[T comparable](name string, tag T) *Counter1[T] {
	return &Counter1[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter1[T]) Push(v T) *Counter1[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter1[T]) Size() int { return len(c.values) }

func (c *Counter1[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter1[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe1(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter2[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter2[T comparable](name string, tag T) *Counter2[T] {
	return &Counter2[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter2[T]) Push(v T) *Counter2[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter2[T]) Size() int { return len(c.values) }

func (c *Counter2[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter2[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe2(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter3[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter3[T comparable](name string, tag T) *Counter3[T] {
	return &Counter3[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter3[T]) Push(v T) *Counter3[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter3[T]) Size() int { return len(c.values) }

func (c *Counter3[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter3[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe3(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter4[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter4[T comparable](name string, tag T) *Counter4[T] {
	return &Counter4[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter4[T]) Push(v T) *Counter4[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter4[T]) Size() int { return len(c.values) }

func (c *Counter4[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter4[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe4(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter5[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter5[T comparable](name string, tag T) *Counter5[T] {
	return &Counter5[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter5[T]) Push(v T) *Counter5[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter5[T]) Size() int { return len(c.values) }

func (c *Counter5[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter5[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe5(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter6[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter6[T comparable](name string, tag T) *Counter6[T] {
	return &Counter6[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter6[T]) Push(v T) *Counter6[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter6[T]) Size() int { return len(c.values) }

func (c *Counter6[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter6[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe6(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter7[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter7[T comparable](name string, tag T) *Counter7[T] {
	return &Counter7[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter7[T]) Push(v T) *Counter7[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter7[T]) Size() int { return len(c.values) }

func (c *Counter7[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter7[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe7(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter8[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter8[T comparable](name string, tag T) *Counter8[T] {
	return &Counter8[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter8[T]) Push(v T) *Counter8[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter8[T]) Size() int { return len(c.values) }

func (c *Counter8[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter8[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe8(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

type Counter9[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter9[T comparable](name string, tag T) *Counter9[T] {
	return &Counter9[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter9[T]) Push(v T) *Counter9[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter9[T]) Size() int { return len(c.values) }

func (c *Counter9[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter9[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe9(v interface{}) string {
	switch x := v.(type) {
	case int:
		return fmt.Sprintf("int:%d", x)
	case string:
		return fmt.Sprintf("str:%q", x)
	case []int:
		return fmt.Sprintf("ints:%d", len(x))
	case map[string]int:
		return fmt.Sprintf("map:%d", len(x))
	default:
		return fmt.Sprintf("other:%T", x)
	}
}

func main() {
	c := NewCounter0[int]("ages", 0)
	data := []int{1, 2, 2, 3, 10, 10, 10, -1}
	for _, v := range data {
		c.Push(v)
	}
	fmt.Println(c.Summary())
	for _, n := range data {
		fmt.Printf("%d -> %s\n", n, classify(n))
	}
}
