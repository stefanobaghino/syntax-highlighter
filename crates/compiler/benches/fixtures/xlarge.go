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

type Counter10[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter10[T comparable](name string, tag T) *Counter10[T] {
	return &Counter10[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter10[T]) Push(v T) *Counter10[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter10[T]) Size() int { return len(c.values) }

func (c *Counter10[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter10[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe10(v interface{}) string {
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

type Counter11[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter11[T comparable](name string, tag T) *Counter11[T] {
	return &Counter11[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter11[T]) Push(v T) *Counter11[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter11[T]) Size() int { return len(c.values) }

func (c *Counter11[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter11[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe11(v interface{}) string {
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

type Counter12[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter12[T comparable](name string, tag T) *Counter12[T] {
	return &Counter12[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter12[T]) Push(v T) *Counter12[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter12[T]) Size() int { return len(c.values) }

func (c *Counter12[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter12[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe12(v interface{}) string {
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

type Counter13[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter13[T comparable](name string, tag T) *Counter13[T] {
	return &Counter13[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter13[T]) Push(v T) *Counter13[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter13[T]) Size() int { return len(c.values) }

func (c *Counter13[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter13[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe13(v interface{}) string {
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

type Counter14[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter14[T comparable](name string, tag T) *Counter14[T] {
	return &Counter14[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter14[T]) Push(v T) *Counter14[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter14[T]) Size() int { return len(c.values) }

func (c *Counter14[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter14[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe14(v interface{}) string {
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

type Counter15[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter15[T comparable](name string, tag T) *Counter15[T] {
	return &Counter15[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter15[T]) Push(v T) *Counter15[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter15[T]) Size() int { return len(c.values) }

func (c *Counter15[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter15[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe15(v interface{}) string {
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

type Counter16[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter16[T comparable](name string, tag T) *Counter16[T] {
	return &Counter16[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter16[T]) Push(v T) *Counter16[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter16[T]) Size() int { return len(c.values) }

func (c *Counter16[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter16[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe16(v interface{}) string {
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

type Counter17[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter17[T comparable](name string, tag T) *Counter17[T] {
	return &Counter17[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter17[T]) Push(v T) *Counter17[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter17[T]) Size() int { return len(c.values) }

func (c *Counter17[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter17[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe17(v interface{}) string {
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

type Counter18[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter18[T comparable](name string, tag T) *Counter18[T] {
	return &Counter18[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter18[T]) Push(v T) *Counter18[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter18[T]) Size() int { return len(c.values) }

func (c *Counter18[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter18[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe18(v interface{}) string {
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

type Counter19[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter19[T comparable](name string, tag T) *Counter19[T] {
	return &Counter19[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter19[T]) Push(v T) *Counter19[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter19[T]) Size() int { return len(c.values) }

func (c *Counter19[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter19[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe19(v interface{}) string {
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

type Counter20[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter20[T comparable](name string, tag T) *Counter20[T] {
	return &Counter20[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter20[T]) Push(v T) *Counter20[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter20[T]) Size() int { return len(c.values) }

func (c *Counter20[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter20[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe20(v interface{}) string {
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

type Counter21[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter21[T comparable](name string, tag T) *Counter21[T] {
	return &Counter21[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter21[T]) Push(v T) *Counter21[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter21[T]) Size() int { return len(c.values) }

func (c *Counter21[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter21[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe21(v interface{}) string {
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

type Counter22[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter22[T comparable](name string, tag T) *Counter22[T] {
	return &Counter22[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter22[T]) Push(v T) *Counter22[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter22[T]) Size() int { return len(c.values) }

func (c *Counter22[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter22[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe22(v interface{}) string {
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

type Counter23[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter23[T comparable](name string, tag T) *Counter23[T] {
	return &Counter23[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter23[T]) Push(v T) *Counter23[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter23[T]) Size() int { return len(c.values) }

func (c *Counter23[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter23[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe23(v interface{}) string {
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

type Counter24[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter24[T comparable](name string, tag T) *Counter24[T] {
	return &Counter24[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter24[T]) Push(v T) *Counter24[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter24[T]) Size() int { return len(c.values) }

func (c *Counter24[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter24[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe24(v interface{}) string {
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

type Counter25[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter25[T comparable](name string, tag T) *Counter25[T] {
	return &Counter25[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter25[T]) Push(v T) *Counter25[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter25[T]) Size() int { return len(c.values) }

func (c *Counter25[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter25[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe25(v interface{}) string {
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

type Counter26[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter26[T comparable](name string, tag T) *Counter26[T] {
	return &Counter26[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter26[T]) Push(v T) *Counter26[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter26[T]) Size() int { return len(c.values) }

func (c *Counter26[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter26[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe26(v interface{}) string {
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

type Counter27[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter27[T comparable](name string, tag T) *Counter27[T] {
	return &Counter27[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter27[T]) Push(v T) *Counter27[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter27[T]) Size() int { return len(c.values) }

func (c *Counter27[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter27[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe27(v interface{}) string {
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

type Counter28[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter28[T comparable](name string, tag T) *Counter28[T] {
	return &Counter28[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter28[T]) Push(v T) *Counter28[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter28[T]) Size() int { return len(c.values) }

func (c *Counter28[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter28[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe28(v interface{}) string {
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

type Counter29[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter29[T comparable](name string, tag T) *Counter29[T] {
	return &Counter29[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter29[T]) Push(v T) *Counter29[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter29[T]) Size() int { return len(c.values) }

func (c *Counter29[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter29[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe29(v interface{}) string {
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

type Counter30[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter30[T comparable](name string, tag T) *Counter30[T] {
	return &Counter30[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter30[T]) Push(v T) *Counter30[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter30[T]) Size() int { return len(c.values) }

func (c *Counter30[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter30[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe30(v interface{}) string {
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

type Counter31[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter31[T comparable](name string, tag T) *Counter31[T] {
	return &Counter31[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter31[T]) Push(v T) *Counter31[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter31[T]) Size() int { return len(c.values) }

func (c *Counter31[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter31[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe31(v interface{}) string {
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

type Counter32[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter32[T comparable](name string, tag T) *Counter32[T] {
	return &Counter32[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter32[T]) Push(v T) *Counter32[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter32[T]) Size() int { return len(c.values) }

func (c *Counter32[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter32[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe32(v interface{}) string {
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

type Counter33[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter33[T comparable](name string, tag T) *Counter33[T] {
	return &Counter33[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter33[T]) Push(v T) *Counter33[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter33[T]) Size() int { return len(c.values) }

func (c *Counter33[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter33[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe33(v interface{}) string {
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

type Counter34[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter34[T comparable](name string, tag T) *Counter34[T] {
	return &Counter34[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter34[T]) Push(v T) *Counter34[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter34[T]) Size() int { return len(c.values) }

func (c *Counter34[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter34[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe34(v interface{}) string {
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

type Counter35[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter35[T comparable](name string, tag T) *Counter35[T] {
	return &Counter35[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter35[T]) Push(v T) *Counter35[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter35[T]) Size() int { return len(c.values) }

func (c *Counter35[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter35[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe35(v interface{}) string {
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

type Counter36[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter36[T comparable](name string, tag T) *Counter36[T] {
	return &Counter36[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter36[T]) Push(v T) *Counter36[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter36[T]) Size() int { return len(c.values) }

func (c *Counter36[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter36[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe36(v interface{}) string {
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

type Counter37[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter37[T comparable](name string, tag T) *Counter37[T] {
	return &Counter37[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter37[T]) Push(v T) *Counter37[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter37[T]) Size() int { return len(c.values) }

func (c *Counter37[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter37[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe37(v interface{}) string {
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

type Counter38[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter38[T comparable](name string, tag T) *Counter38[T] {
	return &Counter38[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter38[T]) Push(v T) *Counter38[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter38[T]) Size() int { return len(c.values) }

func (c *Counter38[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter38[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe38(v interface{}) string {
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

type Counter39[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter39[T comparable](name string, tag T) *Counter39[T] {
	return &Counter39[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter39[T]) Push(v T) *Counter39[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter39[T]) Size() int { return len(c.values) }

func (c *Counter39[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter39[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe39(v interface{}) string {
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

type Counter40[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter40[T comparable](name string, tag T) *Counter40[T] {
	return &Counter40[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter40[T]) Push(v T) *Counter40[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter40[T]) Size() int { return len(c.values) }

func (c *Counter40[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter40[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe40(v interface{}) string {
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

type Counter41[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter41[T comparable](name string, tag T) *Counter41[T] {
	return &Counter41[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter41[T]) Push(v T) *Counter41[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter41[T]) Size() int { return len(c.values) }

func (c *Counter41[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter41[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe41(v interface{}) string {
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

type Counter42[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter42[T comparable](name string, tag T) *Counter42[T] {
	return &Counter42[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter42[T]) Push(v T) *Counter42[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter42[T]) Size() int { return len(c.values) }

func (c *Counter42[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter42[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe42(v interface{}) string {
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

type Counter43[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter43[T comparable](name string, tag T) *Counter43[T] {
	return &Counter43[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter43[T]) Push(v T) *Counter43[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter43[T]) Size() int { return len(c.values) }

func (c *Counter43[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter43[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe43(v interface{}) string {
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

type Counter44[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter44[T comparable](name string, tag T) *Counter44[T] {
	return &Counter44[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter44[T]) Push(v T) *Counter44[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter44[T]) Size() int { return len(c.values) }

func (c *Counter44[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter44[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe44(v interface{}) string {
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

type Counter45[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter45[T comparable](name string, tag T) *Counter45[T] {
	return &Counter45[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter45[T]) Push(v T) *Counter45[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter45[T]) Size() int { return len(c.values) }

func (c *Counter45[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter45[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe45(v interface{}) string {
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

type Counter46[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter46[T comparable](name string, tag T) *Counter46[T] {
	return &Counter46[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter46[T]) Push(v T) *Counter46[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter46[T]) Size() int { return len(c.values) }

func (c *Counter46[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter46[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe46(v interface{}) string {
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

type Counter47[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter47[T comparable](name string, tag T) *Counter47[T] {
	return &Counter47[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter47[T]) Push(v T) *Counter47[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter47[T]) Size() int { return len(c.values) }

func (c *Counter47[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter47[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe47(v interface{}) string {
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

type Counter48[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter48[T comparable](name string, tag T) *Counter48[T] {
	return &Counter48[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter48[T]) Push(v T) *Counter48[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter48[T]) Size() int { return len(c.values) }

func (c *Counter48[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter48[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe48(v interface{}) string {
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

type Counter49[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter49[T comparable](name string, tag T) *Counter49[T] {
	return &Counter49[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter49[T]) Push(v T) *Counter49[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter49[T]) Size() int { return len(c.values) }

func (c *Counter49[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter49[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe49(v interface{}) string {
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

type Counter50[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter50[T comparable](name string, tag T) *Counter50[T] {
	return &Counter50[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter50[T]) Push(v T) *Counter50[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter50[T]) Size() int { return len(c.values) }

func (c *Counter50[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter50[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe50(v interface{}) string {
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

type Counter51[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter51[T comparable](name string, tag T) *Counter51[T] {
	return &Counter51[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter51[T]) Push(v T) *Counter51[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter51[T]) Size() int { return len(c.values) }

func (c *Counter51[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter51[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe51(v interface{}) string {
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

type Counter52[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter52[T comparable](name string, tag T) *Counter52[T] {
	return &Counter52[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter52[T]) Push(v T) *Counter52[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter52[T]) Size() int { return len(c.values) }

func (c *Counter52[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter52[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe52(v interface{}) string {
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

type Counter53[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter53[T comparable](name string, tag T) *Counter53[T] {
	return &Counter53[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter53[T]) Push(v T) *Counter53[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter53[T]) Size() int { return len(c.values) }

func (c *Counter53[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter53[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe53(v interface{}) string {
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

type Counter54[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter54[T comparable](name string, tag T) *Counter54[T] {
	return &Counter54[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter54[T]) Push(v T) *Counter54[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter54[T]) Size() int { return len(c.values) }

func (c *Counter54[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter54[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe54(v interface{}) string {
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

type Counter55[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter55[T comparable](name string, tag T) *Counter55[T] {
	return &Counter55[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter55[T]) Push(v T) *Counter55[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter55[T]) Size() int { return len(c.values) }

func (c *Counter55[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter55[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe55(v interface{}) string {
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

type Counter56[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter56[T comparable](name string, tag T) *Counter56[T] {
	return &Counter56[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter56[T]) Push(v T) *Counter56[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter56[T]) Size() int { return len(c.values) }

func (c *Counter56[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter56[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe56(v interface{}) string {
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

type Counter57[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter57[T comparable](name string, tag T) *Counter57[T] {
	return &Counter57[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter57[T]) Push(v T) *Counter57[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter57[T]) Size() int { return len(c.values) }

func (c *Counter57[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter57[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe57(v interface{}) string {
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

type Counter58[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter58[T comparable](name string, tag T) *Counter58[T] {
	return &Counter58[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter58[T]) Push(v T) *Counter58[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter58[T]) Size() int { return len(c.values) }

func (c *Counter58[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter58[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe58(v interface{}) string {
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

type Counter59[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter59[T comparable](name string, tag T) *Counter59[T] {
	return &Counter59[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter59[T]) Push(v T) *Counter59[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter59[T]) Size() int { return len(c.values) }

func (c *Counter59[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter59[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe59(v interface{}) string {
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

type Counter60[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter60[T comparable](name string, tag T) *Counter60[T] {
	return &Counter60[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter60[T]) Push(v T) *Counter60[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter60[T]) Size() int { return len(c.values) }

func (c *Counter60[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter60[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe60(v interface{}) string {
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

type Counter61[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter61[T comparable](name string, tag T) *Counter61[T] {
	return &Counter61[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter61[T]) Push(v T) *Counter61[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter61[T]) Size() int { return len(c.values) }

func (c *Counter61[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter61[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe61(v interface{}) string {
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

type Counter62[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter62[T comparable](name string, tag T) *Counter62[T] {
	return &Counter62[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter62[T]) Push(v T) *Counter62[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter62[T]) Size() int { return len(c.values) }

func (c *Counter62[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter62[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe62(v interface{}) string {
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

type Counter63[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter63[T comparable](name string, tag T) *Counter63[T] {
	return &Counter63[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter63[T]) Push(v T) *Counter63[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter63[T]) Size() int { return len(c.values) }

func (c *Counter63[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter63[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe63(v interface{}) string {
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

type Counter64[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter64[T comparable](name string, tag T) *Counter64[T] {
	return &Counter64[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter64[T]) Push(v T) *Counter64[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter64[T]) Size() int { return len(c.values) }

func (c *Counter64[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter64[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe64(v interface{}) string {
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

type Counter65[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter65[T comparable](name string, tag T) *Counter65[T] {
	return &Counter65[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter65[T]) Push(v T) *Counter65[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter65[T]) Size() int { return len(c.values) }

func (c *Counter65[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter65[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe65(v interface{}) string {
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

type Counter66[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter66[T comparable](name string, tag T) *Counter66[T] {
	return &Counter66[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter66[T]) Push(v T) *Counter66[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter66[T]) Size() int { return len(c.values) }

func (c *Counter66[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter66[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe66(v interface{}) string {
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

type Counter67[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter67[T comparable](name string, tag T) *Counter67[T] {
	return &Counter67[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter67[T]) Push(v T) *Counter67[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter67[T]) Size() int { return len(c.values) }

func (c *Counter67[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter67[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe67(v interface{}) string {
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

type Counter68[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter68[T comparable](name string, tag T) *Counter68[T] {
	return &Counter68[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter68[T]) Push(v T) *Counter68[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter68[T]) Size() int { return len(c.values) }

func (c *Counter68[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter68[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe68(v interface{}) string {
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

type Counter69[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter69[T comparable](name string, tag T) *Counter69[T] {
	return &Counter69[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter69[T]) Push(v T) *Counter69[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter69[T]) Size() int { return len(c.values) }

func (c *Counter69[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter69[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe69(v interface{}) string {
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

type Counter70[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter70[T comparable](name string, tag T) *Counter70[T] {
	return &Counter70[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter70[T]) Push(v T) *Counter70[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter70[T]) Size() int { return len(c.values) }

func (c *Counter70[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter70[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe70(v interface{}) string {
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

type Counter71[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter71[T comparable](name string, tag T) *Counter71[T] {
	return &Counter71[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter71[T]) Push(v T) *Counter71[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter71[T]) Size() int { return len(c.values) }

func (c *Counter71[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter71[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe71(v interface{}) string {
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

type Counter72[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter72[T comparable](name string, tag T) *Counter72[T] {
	return &Counter72[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter72[T]) Push(v T) *Counter72[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter72[T]) Size() int { return len(c.values) }

func (c *Counter72[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter72[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe72(v interface{}) string {
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

type Counter73[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter73[T comparable](name string, tag T) *Counter73[T] {
	return &Counter73[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter73[T]) Push(v T) *Counter73[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter73[T]) Size() int { return len(c.values) }

func (c *Counter73[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter73[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe73(v interface{}) string {
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

type Counter74[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter74[T comparable](name string, tag T) *Counter74[T] {
	return &Counter74[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter74[T]) Push(v T) *Counter74[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter74[T]) Size() int { return len(c.values) }

func (c *Counter74[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter74[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe74(v interface{}) string {
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

type Counter75[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter75[T comparable](name string, tag T) *Counter75[T] {
	return &Counter75[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter75[T]) Push(v T) *Counter75[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter75[T]) Size() int { return len(c.values) }

func (c *Counter75[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter75[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe75(v interface{}) string {
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

type Counter76[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter76[T comparable](name string, tag T) *Counter76[T] {
	return &Counter76[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter76[T]) Push(v T) *Counter76[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter76[T]) Size() int { return len(c.values) }

func (c *Counter76[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter76[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe76(v interface{}) string {
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

type Counter77[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter77[T comparable](name string, tag T) *Counter77[T] {
	return &Counter77[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter77[T]) Push(v T) *Counter77[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter77[T]) Size() int { return len(c.values) }

func (c *Counter77[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter77[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe77(v interface{}) string {
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

type Counter78[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter78[T comparable](name string, tag T) *Counter78[T] {
	return &Counter78[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter78[T]) Push(v T) *Counter78[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter78[T]) Size() int { return len(c.values) }

func (c *Counter78[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter78[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe78(v interface{}) string {
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

type Counter79[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter79[T comparable](name string, tag T) *Counter79[T] {
	return &Counter79[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter79[T]) Push(v T) *Counter79[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter79[T]) Size() int { return len(c.values) }

func (c *Counter79[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter79[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe79(v interface{}) string {
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

type Counter80[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter80[T comparable](name string, tag T) *Counter80[T] {
	return &Counter80[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter80[T]) Push(v T) *Counter80[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter80[T]) Size() int { return len(c.values) }

func (c *Counter80[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter80[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe80(v interface{}) string {
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

type Counter81[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter81[T comparable](name string, tag T) *Counter81[T] {
	return &Counter81[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter81[T]) Push(v T) *Counter81[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter81[T]) Size() int { return len(c.values) }

func (c *Counter81[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter81[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe81(v interface{}) string {
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

type Counter82[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter82[T comparable](name string, tag T) *Counter82[T] {
	return &Counter82[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter82[T]) Push(v T) *Counter82[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter82[T]) Size() int { return len(c.values) }

func (c *Counter82[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter82[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe82(v interface{}) string {
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

type Counter83[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter83[T comparable](name string, tag T) *Counter83[T] {
	return &Counter83[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter83[T]) Push(v T) *Counter83[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter83[T]) Size() int { return len(c.values) }

func (c *Counter83[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter83[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe83(v interface{}) string {
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

type Counter84[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter84[T comparable](name string, tag T) *Counter84[T] {
	return &Counter84[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter84[T]) Push(v T) *Counter84[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter84[T]) Size() int { return len(c.values) }

func (c *Counter84[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter84[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe84(v interface{}) string {
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

type Counter85[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter85[T comparable](name string, tag T) *Counter85[T] {
	return &Counter85[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter85[T]) Push(v T) *Counter85[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter85[T]) Size() int { return len(c.values) }

func (c *Counter85[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter85[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe85(v interface{}) string {
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

type Counter86[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter86[T comparable](name string, tag T) *Counter86[T] {
	return &Counter86[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter86[T]) Push(v T) *Counter86[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter86[T]) Size() int { return len(c.values) }

func (c *Counter86[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter86[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe86(v interface{}) string {
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

type Counter87[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter87[T comparable](name string, tag T) *Counter87[T] {
	return &Counter87[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter87[T]) Push(v T) *Counter87[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter87[T]) Size() int { return len(c.values) }

func (c *Counter87[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter87[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe87(v interface{}) string {
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

type Counter88[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter88[T comparable](name string, tag T) *Counter88[T] {
	return &Counter88[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter88[T]) Push(v T) *Counter88[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter88[T]) Size() int { return len(c.values) }

func (c *Counter88[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter88[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe88(v interface{}) string {
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

type Counter89[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter89[T comparable](name string, tag T) *Counter89[T] {
	return &Counter89[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter89[T]) Push(v T) *Counter89[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter89[T]) Size() int { return len(c.values) }

func (c *Counter89[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter89[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe89(v interface{}) string {
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

type Counter90[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter90[T comparable](name string, tag T) *Counter90[T] {
	return &Counter90[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter90[T]) Push(v T) *Counter90[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter90[T]) Size() int { return len(c.values) }

func (c *Counter90[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter90[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe90(v interface{}) string {
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

type Counter91[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter91[T comparable](name string, tag T) *Counter91[T] {
	return &Counter91[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter91[T]) Push(v T) *Counter91[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter91[T]) Size() int { return len(c.values) }

func (c *Counter91[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter91[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe91(v interface{}) string {
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

type Counter92[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter92[T comparable](name string, tag T) *Counter92[T] {
	return &Counter92[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter92[T]) Push(v T) *Counter92[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter92[T]) Size() int { return len(c.values) }

func (c *Counter92[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter92[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe92(v interface{}) string {
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

type Counter93[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter93[T comparable](name string, tag T) *Counter93[T] {
	return &Counter93[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter93[T]) Push(v T) *Counter93[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter93[T]) Size() int { return len(c.values) }

func (c *Counter93[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter93[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe93(v interface{}) string {
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

type Counter94[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter94[T comparable](name string, tag T) *Counter94[T] {
	return &Counter94[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter94[T]) Push(v T) *Counter94[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter94[T]) Size() int { return len(c.values) }

func (c *Counter94[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter94[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe94(v interface{}) string {
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

type Counter95[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter95[T comparable](name string, tag T) *Counter95[T] {
	return &Counter95[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter95[T]) Push(v T) *Counter95[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter95[T]) Size() int { return len(c.values) }

func (c *Counter95[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter95[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe95(v interface{}) string {
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

type Counter96[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter96[T comparable](name string, tag T) *Counter96[T] {
	return &Counter96[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter96[T]) Push(v T) *Counter96[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter96[T]) Size() int { return len(c.values) }

func (c *Counter96[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter96[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe96(v interface{}) string {
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

type Counter97[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter97[T comparable](name string, tag T) *Counter97[T] {
	return &Counter97[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter97[T]) Push(v T) *Counter97[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter97[T]) Size() int { return len(c.values) }

func (c *Counter97[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter97[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe97(v interface{}) string {
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

type Counter98[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter98[T comparable](name string, tag T) *Counter98[T] {
	return &Counter98[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter98[T]) Push(v T) *Counter98[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter98[T]) Size() int { return len(c.values) }

func (c *Counter98[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter98[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe98(v interface{}) string {
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

type Counter99[T comparable] struct {
	Name      string  `json:"name"`
	values    []T
	histogram map[string]int
	Tag       T
}

func NewCounter99[T comparable](name string, tag T) *Counter99[T] {
	return &Counter99[T]{
		Name:      name,
		values:    make([]T, 0, 16),
		histogram: map[string]int{},
		Tag:       tag,
	}
}

func (c *Counter99[T]) Push(v T) *Counter99[T] {
	key := fmt.Sprintf("%v", v)
	c.histogram[key]++
	c.values = append(c.values, v)
	return c
}

func (c *Counter99[T]) Size() int { return len(c.values) }

func (c *Counter99[T]) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "%s (n=%d)", c.Name, len(c.values))
	for k, n := range c.histogram {
		fmt.Fprintf(&b, " %s=%d", k, n)
	}
	return b.String()
}

func (c *Counter99[T]) Flush(out chan<- string) {
	defer close(out)
	for k, n := range c.histogram {
		select {
		case out <- fmt.Sprintf("%s=%d", k, n):
		default:
			return
		}
	}
}

func describe99(v interface{}) string {
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
