/* Generated C bench fixture. Not hand-maintained —
 * regenerate via benches/fixtures/gen_c.py if shapes need tweaking.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_VALUES 128
#define DEBUG 1

typedef unsigned int counter_size_t;

enum Kind {
    KIND_INT = 0,
    KIND_STR,
    KIND_BOOL,
    KIND_OTHER,
};

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

int sum_ints(const int *xs, counter_size_t n) {
    int acc = 0;
    for (counter_size_t i = 0; i < n; i++) {
        acc += xs[i];
    }
    return acc;
}

struct Counter0 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter0 *next;
};

typedef struct Counter0 counter0_t;

counter0_t *counter0_new(const char *name, enum Kind kind) {
    counter0_t *c = (counter0_t *)malloc(sizeof(counter0_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter0_push(counter0_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter0_total(const counter0_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter0_dump(const counter0_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter0_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter0_free(counter0_t *c) {
    while (c != NULL) {
        counter0_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter1 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter1 *next;
};

typedef struct Counter1 counter1_t;

counter1_t *counter1_new(const char *name, enum Kind kind) {
    counter1_t *c = (counter1_t *)malloc(sizeof(counter1_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter1_push(counter1_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter1_total(const counter1_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter1_dump(const counter1_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter1_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter1_free(counter1_t *c) {
    while (c != NULL) {
        counter1_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter2 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter2 *next;
};

typedef struct Counter2 counter2_t;

counter2_t *counter2_new(const char *name, enum Kind kind) {
    counter2_t *c = (counter2_t *)malloc(sizeof(counter2_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter2_push(counter2_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter2_total(const counter2_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter2_dump(const counter2_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter2_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter2_free(counter2_t *c) {
    while (c != NULL) {
        counter2_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter3 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter3 *next;
};

typedef struct Counter3 counter3_t;

counter3_t *counter3_new(const char *name, enum Kind kind) {
    counter3_t *c = (counter3_t *)malloc(sizeof(counter3_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter3_push(counter3_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter3_total(const counter3_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter3_dump(const counter3_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter3_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter3_free(counter3_t *c) {
    while (c != NULL) {
        counter3_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter4 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter4 *next;
};

typedef struct Counter4 counter4_t;

counter4_t *counter4_new(const char *name, enum Kind kind) {
    counter4_t *c = (counter4_t *)malloc(sizeof(counter4_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter4_push(counter4_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter4_total(const counter4_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter4_dump(const counter4_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter4_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter4_free(counter4_t *c) {
    while (c != NULL) {
        counter4_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter5 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter5 *next;
};

typedef struct Counter5 counter5_t;

counter5_t *counter5_new(const char *name, enum Kind kind) {
    counter5_t *c = (counter5_t *)malloc(sizeof(counter5_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter5_push(counter5_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter5_total(const counter5_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter5_dump(const counter5_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter5_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter5_free(counter5_t *c) {
    while (c != NULL) {
        counter5_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter6 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter6 *next;
};

typedef struct Counter6 counter6_t;

counter6_t *counter6_new(const char *name, enum Kind kind) {
    counter6_t *c = (counter6_t *)malloc(sizeof(counter6_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter6_push(counter6_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter6_total(const counter6_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter6_dump(const counter6_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter6_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter6_free(counter6_t *c) {
    while (c != NULL) {
        counter6_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter7 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter7 *next;
};

typedef struct Counter7 counter7_t;

counter7_t *counter7_new(const char *name, enum Kind kind) {
    counter7_t *c = (counter7_t *)malloc(sizeof(counter7_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter7_push(counter7_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter7_total(const counter7_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter7_dump(const counter7_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter7_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter7_free(counter7_t *c) {
    while (c != NULL) {
        counter7_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter8 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter8 *next;
};

typedef struct Counter8 counter8_t;

counter8_t *counter8_new(const char *name, enum Kind kind) {
    counter8_t *c = (counter8_t *)malloc(sizeof(counter8_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter8_push(counter8_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter8_total(const counter8_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter8_dump(const counter8_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter8_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter8_free(counter8_t *c) {
    while (c != NULL) {
        counter8_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter9 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter9 *next;
};

typedef struct Counter9 counter9_t;

counter9_t *counter9_new(const char *name, enum Kind kind) {
    counter9_t *c = (counter9_t *)malloc(sizeof(counter9_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter9_push(counter9_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter9_total(const counter9_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter9_dump(const counter9_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter9_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter9_free(counter9_t *c) {
    while (c != NULL) {
        counter9_t *next = c->next;
        free(c);
        c = next;
    }
}

int main(int argc, char **argv) {
    (void)argc;
    (void)argv;
    counter0_t *c = counter0_new("root", KIND_INT);
    int data[] = {1, 2, 2, 3, 10, 10, 10, -1, 50, 200};
    for (int i = 0; i < (int)(sizeof(data) / sizeof(data[0])); i++) {
        counter0_push(c, data[i]);
    }
    counter0_dump(c, stdout);
    counter0_free(c);
    return 0;
}
