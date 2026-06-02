/* Medium-sized C bench fixture: exercises preprocessor directives,
 * struct/enum/typedef, function definitions with pointer/array
 * parameters, switch/for/while, and various literal forms.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_VALUES 64
#define DEBUG 1

typedef unsigned int counter_size_t;

enum Kind {
    KIND_INT = 0,
    KIND_STR,
    KIND_BOOL,
};

struct Counter {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
};

struct Counter *counter_new(const char *name, enum Kind kind) {
    struct Counter *c = (struct Counter *)malloc(sizeof(struct Counter));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    return c;
}

int counter_push(struct Counter *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

const char *classify(int n) {
    switch (n) {
        case 0: return "zero";
        case 1:
        case 2:
        case 3:
            return "small";
        default:
            if (n < 0) return "negative";
            if (n < 100) return "medium";
            return "large";
    }
}

int sum(const int *xs, counter_size_t n) {
    int acc = 0;
    for (counter_size_t i = 0; i < n; i++) {
        acc += xs[i];
    }
    return acc;
}

int main(int argc, char **argv) {
    struct Counter *c = counter_new("ages", KIND_INT);
    int data[] = {1, 2, 2, 3, 10, 10, 10, -1};
    for (int i = 0; i < (int)(sizeof(data) / sizeof(data[0])); i++) {
        counter_push(c, data[i]);
    }
    printf("%s (n=%u, total=%d)\n", c->name, c->len, sum(c->values, c->len));
    for (int i = 0; i < (int)c->len; i++) {
        printf("%d -> %s\n", c->values[i], classify(c->values[i]));
    }
    free(c);
    return 0;
}
