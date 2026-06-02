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
struct Counter10 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter10 *next;
};

typedef struct Counter10 counter10_t;

counter10_t *counter10_new(const char *name, enum Kind kind) {
    counter10_t *c = (counter10_t *)malloc(sizeof(counter10_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter10_push(counter10_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter10_total(const counter10_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter10_dump(const counter10_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter10_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter10_free(counter10_t *c) {
    while (c != NULL) {
        counter10_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter11 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter11 *next;
};

typedef struct Counter11 counter11_t;

counter11_t *counter11_new(const char *name, enum Kind kind) {
    counter11_t *c = (counter11_t *)malloc(sizeof(counter11_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter11_push(counter11_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter11_total(const counter11_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter11_dump(const counter11_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter11_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter11_free(counter11_t *c) {
    while (c != NULL) {
        counter11_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter12 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter12 *next;
};

typedef struct Counter12 counter12_t;

counter12_t *counter12_new(const char *name, enum Kind kind) {
    counter12_t *c = (counter12_t *)malloc(sizeof(counter12_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter12_push(counter12_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter12_total(const counter12_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter12_dump(const counter12_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter12_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter12_free(counter12_t *c) {
    while (c != NULL) {
        counter12_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter13 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter13 *next;
};

typedef struct Counter13 counter13_t;

counter13_t *counter13_new(const char *name, enum Kind kind) {
    counter13_t *c = (counter13_t *)malloc(sizeof(counter13_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter13_push(counter13_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter13_total(const counter13_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter13_dump(const counter13_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter13_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter13_free(counter13_t *c) {
    while (c != NULL) {
        counter13_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter14 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter14 *next;
};

typedef struct Counter14 counter14_t;

counter14_t *counter14_new(const char *name, enum Kind kind) {
    counter14_t *c = (counter14_t *)malloc(sizeof(counter14_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter14_push(counter14_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter14_total(const counter14_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter14_dump(const counter14_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter14_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter14_free(counter14_t *c) {
    while (c != NULL) {
        counter14_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter15 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter15 *next;
};

typedef struct Counter15 counter15_t;

counter15_t *counter15_new(const char *name, enum Kind kind) {
    counter15_t *c = (counter15_t *)malloc(sizeof(counter15_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter15_push(counter15_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter15_total(const counter15_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter15_dump(const counter15_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter15_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter15_free(counter15_t *c) {
    while (c != NULL) {
        counter15_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter16 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter16 *next;
};

typedef struct Counter16 counter16_t;

counter16_t *counter16_new(const char *name, enum Kind kind) {
    counter16_t *c = (counter16_t *)malloc(sizeof(counter16_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter16_push(counter16_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter16_total(const counter16_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter16_dump(const counter16_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter16_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter16_free(counter16_t *c) {
    while (c != NULL) {
        counter16_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter17 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter17 *next;
};

typedef struct Counter17 counter17_t;

counter17_t *counter17_new(const char *name, enum Kind kind) {
    counter17_t *c = (counter17_t *)malloc(sizeof(counter17_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter17_push(counter17_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter17_total(const counter17_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter17_dump(const counter17_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter17_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter17_free(counter17_t *c) {
    while (c != NULL) {
        counter17_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter18 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter18 *next;
};

typedef struct Counter18 counter18_t;

counter18_t *counter18_new(const char *name, enum Kind kind) {
    counter18_t *c = (counter18_t *)malloc(sizeof(counter18_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter18_push(counter18_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter18_total(const counter18_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter18_dump(const counter18_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter18_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter18_free(counter18_t *c) {
    while (c != NULL) {
        counter18_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter19 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter19 *next;
};

typedef struct Counter19 counter19_t;

counter19_t *counter19_new(const char *name, enum Kind kind) {
    counter19_t *c = (counter19_t *)malloc(sizeof(counter19_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter19_push(counter19_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter19_total(const counter19_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter19_dump(const counter19_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter19_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter19_free(counter19_t *c) {
    while (c != NULL) {
        counter19_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter20 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter20 *next;
};

typedef struct Counter20 counter20_t;

counter20_t *counter20_new(const char *name, enum Kind kind) {
    counter20_t *c = (counter20_t *)malloc(sizeof(counter20_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter20_push(counter20_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter20_total(const counter20_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter20_dump(const counter20_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter20_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter20_free(counter20_t *c) {
    while (c != NULL) {
        counter20_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter21 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter21 *next;
};

typedef struct Counter21 counter21_t;

counter21_t *counter21_new(const char *name, enum Kind kind) {
    counter21_t *c = (counter21_t *)malloc(sizeof(counter21_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter21_push(counter21_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter21_total(const counter21_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter21_dump(const counter21_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter21_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter21_free(counter21_t *c) {
    while (c != NULL) {
        counter21_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter22 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter22 *next;
};

typedef struct Counter22 counter22_t;

counter22_t *counter22_new(const char *name, enum Kind kind) {
    counter22_t *c = (counter22_t *)malloc(sizeof(counter22_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter22_push(counter22_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter22_total(const counter22_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter22_dump(const counter22_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter22_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter22_free(counter22_t *c) {
    while (c != NULL) {
        counter22_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter23 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter23 *next;
};

typedef struct Counter23 counter23_t;

counter23_t *counter23_new(const char *name, enum Kind kind) {
    counter23_t *c = (counter23_t *)malloc(sizeof(counter23_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter23_push(counter23_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter23_total(const counter23_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter23_dump(const counter23_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter23_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter23_free(counter23_t *c) {
    while (c != NULL) {
        counter23_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter24 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter24 *next;
};

typedef struct Counter24 counter24_t;

counter24_t *counter24_new(const char *name, enum Kind kind) {
    counter24_t *c = (counter24_t *)malloc(sizeof(counter24_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter24_push(counter24_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter24_total(const counter24_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter24_dump(const counter24_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter24_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter24_free(counter24_t *c) {
    while (c != NULL) {
        counter24_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter25 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter25 *next;
};

typedef struct Counter25 counter25_t;

counter25_t *counter25_new(const char *name, enum Kind kind) {
    counter25_t *c = (counter25_t *)malloc(sizeof(counter25_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter25_push(counter25_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter25_total(const counter25_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter25_dump(const counter25_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter25_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter25_free(counter25_t *c) {
    while (c != NULL) {
        counter25_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter26 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter26 *next;
};

typedef struct Counter26 counter26_t;

counter26_t *counter26_new(const char *name, enum Kind kind) {
    counter26_t *c = (counter26_t *)malloc(sizeof(counter26_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter26_push(counter26_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter26_total(const counter26_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter26_dump(const counter26_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter26_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter26_free(counter26_t *c) {
    while (c != NULL) {
        counter26_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter27 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter27 *next;
};

typedef struct Counter27 counter27_t;

counter27_t *counter27_new(const char *name, enum Kind kind) {
    counter27_t *c = (counter27_t *)malloc(sizeof(counter27_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter27_push(counter27_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter27_total(const counter27_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter27_dump(const counter27_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter27_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter27_free(counter27_t *c) {
    while (c != NULL) {
        counter27_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter28 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter28 *next;
};

typedef struct Counter28 counter28_t;

counter28_t *counter28_new(const char *name, enum Kind kind) {
    counter28_t *c = (counter28_t *)malloc(sizeof(counter28_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter28_push(counter28_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter28_total(const counter28_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter28_dump(const counter28_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter28_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter28_free(counter28_t *c) {
    while (c != NULL) {
        counter28_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter29 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter29 *next;
};

typedef struct Counter29 counter29_t;

counter29_t *counter29_new(const char *name, enum Kind kind) {
    counter29_t *c = (counter29_t *)malloc(sizeof(counter29_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter29_push(counter29_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter29_total(const counter29_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter29_dump(const counter29_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter29_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter29_free(counter29_t *c) {
    while (c != NULL) {
        counter29_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter30 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter30 *next;
};

typedef struct Counter30 counter30_t;

counter30_t *counter30_new(const char *name, enum Kind kind) {
    counter30_t *c = (counter30_t *)malloc(sizeof(counter30_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter30_push(counter30_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter30_total(const counter30_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter30_dump(const counter30_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter30_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter30_free(counter30_t *c) {
    while (c != NULL) {
        counter30_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter31 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter31 *next;
};

typedef struct Counter31 counter31_t;

counter31_t *counter31_new(const char *name, enum Kind kind) {
    counter31_t *c = (counter31_t *)malloc(sizeof(counter31_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter31_push(counter31_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter31_total(const counter31_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter31_dump(const counter31_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter31_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter31_free(counter31_t *c) {
    while (c != NULL) {
        counter31_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter32 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter32 *next;
};

typedef struct Counter32 counter32_t;

counter32_t *counter32_new(const char *name, enum Kind kind) {
    counter32_t *c = (counter32_t *)malloc(sizeof(counter32_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter32_push(counter32_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter32_total(const counter32_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter32_dump(const counter32_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter32_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter32_free(counter32_t *c) {
    while (c != NULL) {
        counter32_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter33 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter33 *next;
};

typedef struct Counter33 counter33_t;

counter33_t *counter33_new(const char *name, enum Kind kind) {
    counter33_t *c = (counter33_t *)malloc(sizeof(counter33_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter33_push(counter33_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter33_total(const counter33_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter33_dump(const counter33_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter33_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter33_free(counter33_t *c) {
    while (c != NULL) {
        counter33_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter34 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter34 *next;
};

typedef struct Counter34 counter34_t;

counter34_t *counter34_new(const char *name, enum Kind kind) {
    counter34_t *c = (counter34_t *)malloc(sizeof(counter34_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter34_push(counter34_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter34_total(const counter34_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter34_dump(const counter34_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter34_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter34_free(counter34_t *c) {
    while (c != NULL) {
        counter34_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter35 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter35 *next;
};

typedef struct Counter35 counter35_t;

counter35_t *counter35_new(const char *name, enum Kind kind) {
    counter35_t *c = (counter35_t *)malloc(sizeof(counter35_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter35_push(counter35_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter35_total(const counter35_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter35_dump(const counter35_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter35_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter35_free(counter35_t *c) {
    while (c != NULL) {
        counter35_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter36 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter36 *next;
};

typedef struct Counter36 counter36_t;

counter36_t *counter36_new(const char *name, enum Kind kind) {
    counter36_t *c = (counter36_t *)malloc(sizeof(counter36_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter36_push(counter36_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter36_total(const counter36_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter36_dump(const counter36_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter36_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter36_free(counter36_t *c) {
    while (c != NULL) {
        counter36_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter37 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter37 *next;
};

typedef struct Counter37 counter37_t;

counter37_t *counter37_new(const char *name, enum Kind kind) {
    counter37_t *c = (counter37_t *)malloc(sizeof(counter37_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter37_push(counter37_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter37_total(const counter37_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter37_dump(const counter37_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter37_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter37_free(counter37_t *c) {
    while (c != NULL) {
        counter37_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter38 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter38 *next;
};

typedef struct Counter38 counter38_t;

counter38_t *counter38_new(const char *name, enum Kind kind) {
    counter38_t *c = (counter38_t *)malloc(sizeof(counter38_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter38_push(counter38_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter38_total(const counter38_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter38_dump(const counter38_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter38_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter38_free(counter38_t *c) {
    while (c != NULL) {
        counter38_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter39 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter39 *next;
};

typedef struct Counter39 counter39_t;

counter39_t *counter39_new(const char *name, enum Kind kind) {
    counter39_t *c = (counter39_t *)malloc(sizeof(counter39_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter39_push(counter39_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter39_total(const counter39_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter39_dump(const counter39_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter39_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter39_free(counter39_t *c) {
    while (c != NULL) {
        counter39_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter40 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter40 *next;
};

typedef struct Counter40 counter40_t;

counter40_t *counter40_new(const char *name, enum Kind kind) {
    counter40_t *c = (counter40_t *)malloc(sizeof(counter40_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter40_push(counter40_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter40_total(const counter40_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter40_dump(const counter40_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter40_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter40_free(counter40_t *c) {
    while (c != NULL) {
        counter40_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter41 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter41 *next;
};

typedef struct Counter41 counter41_t;

counter41_t *counter41_new(const char *name, enum Kind kind) {
    counter41_t *c = (counter41_t *)malloc(sizeof(counter41_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter41_push(counter41_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter41_total(const counter41_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter41_dump(const counter41_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter41_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter41_free(counter41_t *c) {
    while (c != NULL) {
        counter41_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter42 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter42 *next;
};

typedef struct Counter42 counter42_t;

counter42_t *counter42_new(const char *name, enum Kind kind) {
    counter42_t *c = (counter42_t *)malloc(sizeof(counter42_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter42_push(counter42_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter42_total(const counter42_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter42_dump(const counter42_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter42_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter42_free(counter42_t *c) {
    while (c != NULL) {
        counter42_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter43 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter43 *next;
};

typedef struct Counter43 counter43_t;

counter43_t *counter43_new(const char *name, enum Kind kind) {
    counter43_t *c = (counter43_t *)malloc(sizeof(counter43_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter43_push(counter43_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter43_total(const counter43_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter43_dump(const counter43_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter43_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter43_free(counter43_t *c) {
    while (c != NULL) {
        counter43_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter44 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter44 *next;
};

typedef struct Counter44 counter44_t;

counter44_t *counter44_new(const char *name, enum Kind kind) {
    counter44_t *c = (counter44_t *)malloc(sizeof(counter44_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter44_push(counter44_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter44_total(const counter44_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter44_dump(const counter44_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter44_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter44_free(counter44_t *c) {
    while (c != NULL) {
        counter44_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter45 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter45 *next;
};

typedef struct Counter45 counter45_t;

counter45_t *counter45_new(const char *name, enum Kind kind) {
    counter45_t *c = (counter45_t *)malloc(sizeof(counter45_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter45_push(counter45_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter45_total(const counter45_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter45_dump(const counter45_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter45_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter45_free(counter45_t *c) {
    while (c != NULL) {
        counter45_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter46 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter46 *next;
};

typedef struct Counter46 counter46_t;

counter46_t *counter46_new(const char *name, enum Kind kind) {
    counter46_t *c = (counter46_t *)malloc(sizeof(counter46_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter46_push(counter46_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter46_total(const counter46_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter46_dump(const counter46_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter46_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter46_free(counter46_t *c) {
    while (c != NULL) {
        counter46_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter47 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter47 *next;
};

typedef struct Counter47 counter47_t;

counter47_t *counter47_new(const char *name, enum Kind kind) {
    counter47_t *c = (counter47_t *)malloc(sizeof(counter47_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter47_push(counter47_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter47_total(const counter47_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter47_dump(const counter47_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter47_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter47_free(counter47_t *c) {
    while (c != NULL) {
        counter47_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter48 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter48 *next;
};

typedef struct Counter48 counter48_t;

counter48_t *counter48_new(const char *name, enum Kind kind) {
    counter48_t *c = (counter48_t *)malloc(sizeof(counter48_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter48_push(counter48_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter48_total(const counter48_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter48_dump(const counter48_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter48_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter48_free(counter48_t *c) {
    while (c != NULL) {
        counter48_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter49 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter49 *next;
};

typedef struct Counter49 counter49_t;

counter49_t *counter49_new(const char *name, enum Kind kind) {
    counter49_t *c = (counter49_t *)malloc(sizeof(counter49_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter49_push(counter49_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter49_total(const counter49_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter49_dump(const counter49_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter49_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter49_free(counter49_t *c) {
    while (c != NULL) {
        counter49_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter50 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter50 *next;
};

typedef struct Counter50 counter50_t;

counter50_t *counter50_new(const char *name, enum Kind kind) {
    counter50_t *c = (counter50_t *)malloc(sizeof(counter50_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter50_push(counter50_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter50_total(const counter50_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter50_dump(const counter50_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter50_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter50_free(counter50_t *c) {
    while (c != NULL) {
        counter50_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter51 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter51 *next;
};

typedef struct Counter51 counter51_t;

counter51_t *counter51_new(const char *name, enum Kind kind) {
    counter51_t *c = (counter51_t *)malloc(sizeof(counter51_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter51_push(counter51_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter51_total(const counter51_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter51_dump(const counter51_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter51_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter51_free(counter51_t *c) {
    while (c != NULL) {
        counter51_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter52 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter52 *next;
};

typedef struct Counter52 counter52_t;

counter52_t *counter52_new(const char *name, enum Kind kind) {
    counter52_t *c = (counter52_t *)malloc(sizeof(counter52_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter52_push(counter52_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter52_total(const counter52_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter52_dump(const counter52_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter52_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter52_free(counter52_t *c) {
    while (c != NULL) {
        counter52_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter53 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter53 *next;
};

typedef struct Counter53 counter53_t;

counter53_t *counter53_new(const char *name, enum Kind kind) {
    counter53_t *c = (counter53_t *)malloc(sizeof(counter53_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter53_push(counter53_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter53_total(const counter53_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter53_dump(const counter53_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter53_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter53_free(counter53_t *c) {
    while (c != NULL) {
        counter53_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter54 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter54 *next;
};

typedef struct Counter54 counter54_t;

counter54_t *counter54_new(const char *name, enum Kind kind) {
    counter54_t *c = (counter54_t *)malloc(sizeof(counter54_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter54_push(counter54_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter54_total(const counter54_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter54_dump(const counter54_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter54_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter54_free(counter54_t *c) {
    while (c != NULL) {
        counter54_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter55 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter55 *next;
};

typedef struct Counter55 counter55_t;

counter55_t *counter55_new(const char *name, enum Kind kind) {
    counter55_t *c = (counter55_t *)malloc(sizeof(counter55_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter55_push(counter55_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter55_total(const counter55_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter55_dump(const counter55_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter55_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter55_free(counter55_t *c) {
    while (c != NULL) {
        counter55_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter56 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter56 *next;
};

typedef struct Counter56 counter56_t;

counter56_t *counter56_new(const char *name, enum Kind kind) {
    counter56_t *c = (counter56_t *)malloc(sizeof(counter56_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter56_push(counter56_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter56_total(const counter56_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter56_dump(const counter56_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter56_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter56_free(counter56_t *c) {
    while (c != NULL) {
        counter56_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter57 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter57 *next;
};

typedef struct Counter57 counter57_t;

counter57_t *counter57_new(const char *name, enum Kind kind) {
    counter57_t *c = (counter57_t *)malloc(sizeof(counter57_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter57_push(counter57_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter57_total(const counter57_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter57_dump(const counter57_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter57_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter57_free(counter57_t *c) {
    while (c != NULL) {
        counter57_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter58 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter58 *next;
};

typedef struct Counter58 counter58_t;

counter58_t *counter58_new(const char *name, enum Kind kind) {
    counter58_t *c = (counter58_t *)malloc(sizeof(counter58_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter58_push(counter58_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter58_total(const counter58_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter58_dump(const counter58_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter58_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter58_free(counter58_t *c) {
    while (c != NULL) {
        counter58_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter59 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter59 *next;
};

typedef struct Counter59 counter59_t;

counter59_t *counter59_new(const char *name, enum Kind kind) {
    counter59_t *c = (counter59_t *)malloc(sizeof(counter59_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter59_push(counter59_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter59_total(const counter59_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter59_dump(const counter59_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter59_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter59_free(counter59_t *c) {
    while (c != NULL) {
        counter59_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter60 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter60 *next;
};

typedef struct Counter60 counter60_t;

counter60_t *counter60_new(const char *name, enum Kind kind) {
    counter60_t *c = (counter60_t *)malloc(sizeof(counter60_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter60_push(counter60_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter60_total(const counter60_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter60_dump(const counter60_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter60_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter60_free(counter60_t *c) {
    while (c != NULL) {
        counter60_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter61 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter61 *next;
};

typedef struct Counter61 counter61_t;

counter61_t *counter61_new(const char *name, enum Kind kind) {
    counter61_t *c = (counter61_t *)malloc(sizeof(counter61_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter61_push(counter61_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter61_total(const counter61_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter61_dump(const counter61_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter61_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter61_free(counter61_t *c) {
    while (c != NULL) {
        counter61_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter62 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter62 *next;
};

typedef struct Counter62 counter62_t;

counter62_t *counter62_new(const char *name, enum Kind kind) {
    counter62_t *c = (counter62_t *)malloc(sizeof(counter62_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter62_push(counter62_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter62_total(const counter62_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter62_dump(const counter62_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter62_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter62_free(counter62_t *c) {
    while (c != NULL) {
        counter62_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter63 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter63 *next;
};

typedef struct Counter63 counter63_t;

counter63_t *counter63_new(const char *name, enum Kind kind) {
    counter63_t *c = (counter63_t *)malloc(sizeof(counter63_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter63_push(counter63_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter63_total(const counter63_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter63_dump(const counter63_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter63_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter63_free(counter63_t *c) {
    while (c != NULL) {
        counter63_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter64 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter64 *next;
};

typedef struct Counter64 counter64_t;

counter64_t *counter64_new(const char *name, enum Kind kind) {
    counter64_t *c = (counter64_t *)malloc(sizeof(counter64_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter64_push(counter64_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter64_total(const counter64_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter64_dump(const counter64_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter64_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter64_free(counter64_t *c) {
    while (c != NULL) {
        counter64_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter65 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter65 *next;
};

typedef struct Counter65 counter65_t;

counter65_t *counter65_new(const char *name, enum Kind kind) {
    counter65_t *c = (counter65_t *)malloc(sizeof(counter65_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter65_push(counter65_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter65_total(const counter65_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter65_dump(const counter65_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter65_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter65_free(counter65_t *c) {
    while (c != NULL) {
        counter65_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter66 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter66 *next;
};

typedef struct Counter66 counter66_t;

counter66_t *counter66_new(const char *name, enum Kind kind) {
    counter66_t *c = (counter66_t *)malloc(sizeof(counter66_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter66_push(counter66_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter66_total(const counter66_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter66_dump(const counter66_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter66_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter66_free(counter66_t *c) {
    while (c != NULL) {
        counter66_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter67 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter67 *next;
};

typedef struct Counter67 counter67_t;

counter67_t *counter67_new(const char *name, enum Kind kind) {
    counter67_t *c = (counter67_t *)malloc(sizeof(counter67_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter67_push(counter67_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter67_total(const counter67_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter67_dump(const counter67_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter67_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter67_free(counter67_t *c) {
    while (c != NULL) {
        counter67_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter68 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter68 *next;
};

typedef struct Counter68 counter68_t;

counter68_t *counter68_new(const char *name, enum Kind kind) {
    counter68_t *c = (counter68_t *)malloc(sizeof(counter68_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter68_push(counter68_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter68_total(const counter68_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter68_dump(const counter68_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter68_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter68_free(counter68_t *c) {
    while (c != NULL) {
        counter68_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter69 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter69 *next;
};

typedef struct Counter69 counter69_t;

counter69_t *counter69_new(const char *name, enum Kind kind) {
    counter69_t *c = (counter69_t *)malloc(sizeof(counter69_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter69_push(counter69_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter69_total(const counter69_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter69_dump(const counter69_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter69_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter69_free(counter69_t *c) {
    while (c != NULL) {
        counter69_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter70 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter70 *next;
};

typedef struct Counter70 counter70_t;

counter70_t *counter70_new(const char *name, enum Kind kind) {
    counter70_t *c = (counter70_t *)malloc(sizeof(counter70_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter70_push(counter70_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter70_total(const counter70_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter70_dump(const counter70_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter70_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter70_free(counter70_t *c) {
    while (c != NULL) {
        counter70_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter71 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter71 *next;
};

typedef struct Counter71 counter71_t;

counter71_t *counter71_new(const char *name, enum Kind kind) {
    counter71_t *c = (counter71_t *)malloc(sizeof(counter71_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter71_push(counter71_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter71_total(const counter71_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter71_dump(const counter71_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter71_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter71_free(counter71_t *c) {
    while (c != NULL) {
        counter71_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter72 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter72 *next;
};

typedef struct Counter72 counter72_t;

counter72_t *counter72_new(const char *name, enum Kind kind) {
    counter72_t *c = (counter72_t *)malloc(sizeof(counter72_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter72_push(counter72_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter72_total(const counter72_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter72_dump(const counter72_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter72_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter72_free(counter72_t *c) {
    while (c != NULL) {
        counter72_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter73 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter73 *next;
};

typedef struct Counter73 counter73_t;

counter73_t *counter73_new(const char *name, enum Kind kind) {
    counter73_t *c = (counter73_t *)malloc(sizeof(counter73_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter73_push(counter73_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter73_total(const counter73_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter73_dump(const counter73_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter73_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter73_free(counter73_t *c) {
    while (c != NULL) {
        counter73_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter74 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter74 *next;
};

typedef struct Counter74 counter74_t;

counter74_t *counter74_new(const char *name, enum Kind kind) {
    counter74_t *c = (counter74_t *)malloc(sizeof(counter74_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter74_push(counter74_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter74_total(const counter74_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter74_dump(const counter74_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter74_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter74_free(counter74_t *c) {
    while (c != NULL) {
        counter74_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter75 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter75 *next;
};

typedef struct Counter75 counter75_t;

counter75_t *counter75_new(const char *name, enum Kind kind) {
    counter75_t *c = (counter75_t *)malloc(sizeof(counter75_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter75_push(counter75_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter75_total(const counter75_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter75_dump(const counter75_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter75_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter75_free(counter75_t *c) {
    while (c != NULL) {
        counter75_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter76 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter76 *next;
};

typedef struct Counter76 counter76_t;

counter76_t *counter76_new(const char *name, enum Kind kind) {
    counter76_t *c = (counter76_t *)malloc(sizeof(counter76_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter76_push(counter76_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter76_total(const counter76_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter76_dump(const counter76_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter76_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter76_free(counter76_t *c) {
    while (c != NULL) {
        counter76_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter77 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter77 *next;
};

typedef struct Counter77 counter77_t;

counter77_t *counter77_new(const char *name, enum Kind kind) {
    counter77_t *c = (counter77_t *)malloc(sizeof(counter77_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter77_push(counter77_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter77_total(const counter77_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter77_dump(const counter77_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter77_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter77_free(counter77_t *c) {
    while (c != NULL) {
        counter77_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter78 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter78 *next;
};

typedef struct Counter78 counter78_t;

counter78_t *counter78_new(const char *name, enum Kind kind) {
    counter78_t *c = (counter78_t *)malloc(sizeof(counter78_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter78_push(counter78_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter78_total(const counter78_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter78_dump(const counter78_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter78_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter78_free(counter78_t *c) {
    while (c != NULL) {
        counter78_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter79 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter79 *next;
};

typedef struct Counter79 counter79_t;

counter79_t *counter79_new(const char *name, enum Kind kind) {
    counter79_t *c = (counter79_t *)malloc(sizeof(counter79_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter79_push(counter79_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter79_total(const counter79_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter79_dump(const counter79_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter79_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter79_free(counter79_t *c) {
    while (c != NULL) {
        counter79_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter80 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter80 *next;
};

typedef struct Counter80 counter80_t;

counter80_t *counter80_new(const char *name, enum Kind kind) {
    counter80_t *c = (counter80_t *)malloc(sizeof(counter80_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter80_push(counter80_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter80_total(const counter80_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter80_dump(const counter80_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter80_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter80_free(counter80_t *c) {
    while (c != NULL) {
        counter80_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter81 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter81 *next;
};

typedef struct Counter81 counter81_t;

counter81_t *counter81_new(const char *name, enum Kind kind) {
    counter81_t *c = (counter81_t *)malloc(sizeof(counter81_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter81_push(counter81_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter81_total(const counter81_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter81_dump(const counter81_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter81_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter81_free(counter81_t *c) {
    while (c != NULL) {
        counter81_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter82 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter82 *next;
};

typedef struct Counter82 counter82_t;

counter82_t *counter82_new(const char *name, enum Kind kind) {
    counter82_t *c = (counter82_t *)malloc(sizeof(counter82_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter82_push(counter82_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter82_total(const counter82_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter82_dump(const counter82_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter82_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter82_free(counter82_t *c) {
    while (c != NULL) {
        counter82_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter83 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter83 *next;
};

typedef struct Counter83 counter83_t;

counter83_t *counter83_new(const char *name, enum Kind kind) {
    counter83_t *c = (counter83_t *)malloc(sizeof(counter83_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter83_push(counter83_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter83_total(const counter83_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter83_dump(const counter83_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter83_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter83_free(counter83_t *c) {
    while (c != NULL) {
        counter83_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter84 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter84 *next;
};

typedef struct Counter84 counter84_t;

counter84_t *counter84_new(const char *name, enum Kind kind) {
    counter84_t *c = (counter84_t *)malloc(sizeof(counter84_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter84_push(counter84_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter84_total(const counter84_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter84_dump(const counter84_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter84_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter84_free(counter84_t *c) {
    while (c != NULL) {
        counter84_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter85 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter85 *next;
};

typedef struct Counter85 counter85_t;

counter85_t *counter85_new(const char *name, enum Kind kind) {
    counter85_t *c = (counter85_t *)malloc(sizeof(counter85_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter85_push(counter85_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter85_total(const counter85_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter85_dump(const counter85_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter85_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter85_free(counter85_t *c) {
    while (c != NULL) {
        counter85_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter86 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter86 *next;
};

typedef struct Counter86 counter86_t;

counter86_t *counter86_new(const char *name, enum Kind kind) {
    counter86_t *c = (counter86_t *)malloc(sizeof(counter86_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter86_push(counter86_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter86_total(const counter86_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter86_dump(const counter86_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter86_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter86_free(counter86_t *c) {
    while (c != NULL) {
        counter86_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter87 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter87 *next;
};

typedef struct Counter87 counter87_t;

counter87_t *counter87_new(const char *name, enum Kind kind) {
    counter87_t *c = (counter87_t *)malloc(sizeof(counter87_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter87_push(counter87_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter87_total(const counter87_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter87_dump(const counter87_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter87_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter87_free(counter87_t *c) {
    while (c != NULL) {
        counter87_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter88 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter88 *next;
};

typedef struct Counter88 counter88_t;

counter88_t *counter88_new(const char *name, enum Kind kind) {
    counter88_t *c = (counter88_t *)malloc(sizeof(counter88_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter88_push(counter88_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter88_total(const counter88_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter88_dump(const counter88_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter88_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter88_free(counter88_t *c) {
    while (c != NULL) {
        counter88_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter89 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter89 *next;
};

typedef struct Counter89 counter89_t;

counter89_t *counter89_new(const char *name, enum Kind kind) {
    counter89_t *c = (counter89_t *)malloc(sizeof(counter89_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter89_push(counter89_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter89_total(const counter89_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter89_dump(const counter89_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter89_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter89_free(counter89_t *c) {
    while (c != NULL) {
        counter89_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter90 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter90 *next;
};

typedef struct Counter90 counter90_t;

counter90_t *counter90_new(const char *name, enum Kind kind) {
    counter90_t *c = (counter90_t *)malloc(sizeof(counter90_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter90_push(counter90_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter90_total(const counter90_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter90_dump(const counter90_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter90_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter90_free(counter90_t *c) {
    while (c != NULL) {
        counter90_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter91 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter91 *next;
};

typedef struct Counter91 counter91_t;

counter91_t *counter91_new(const char *name, enum Kind kind) {
    counter91_t *c = (counter91_t *)malloc(sizeof(counter91_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter91_push(counter91_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter91_total(const counter91_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter91_dump(const counter91_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter91_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter91_free(counter91_t *c) {
    while (c != NULL) {
        counter91_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter92 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter92 *next;
};

typedef struct Counter92 counter92_t;

counter92_t *counter92_new(const char *name, enum Kind kind) {
    counter92_t *c = (counter92_t *)malloc(sizeof(counter92_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter92_push(counter92_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter92_total(const counter92_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter92_dump(const counter92_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter92_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter92_free(counter92_t *c) {
    while (c != NULL) {
        counter92_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter93 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter93 *next;
};

typedef struct Counter93 counter93_t;

counter93_t *counter93_new(const char *name, enum Kind kind) {
    counter93_t *c = (counter93_t *)malloc(sizeof(counter93_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter93_push(counter93_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter93_total(const counter93_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter93_dump(const counter93_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter93_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter93_free(counter93_t *c) {
    while (c != NULL) {
        counter93_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter94 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter94 *next;
};

typedef struct Counter94 counter94_t;

counter94_t *counter94_new(const char *name, enum Kind kind) {
    counter94_t *c = (counter94_t *)malloc(sizeof(counter94_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter94_push(counter94_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter94_total(const counter94_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter94_dump(const counter94_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter94_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter94_free(counter94_t *c) {
    while (c != NULL) {
        counter94_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter95 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter95 *next;
};

typedef struct Counter95 counter95_t;

counter95_t *counter95_new(const char *name, enum Kind kind) {
    counter95_t *c = (counter95_t *)malloc(sizeof(counter95_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter95_push(counter95_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter95_total(const counter95_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter95_dump(const counter95_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter95_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter95_free(counter95_t *c) {
    while (c != NULL) {
        counter95_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter96 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter96 *next;
};

typedef struct Counter96 counter96_t;

counter96_t *counter96_new(const char *name, enum Kind kind) {
    counter96_t *c = (counter96_t *)malloc(sizeof(counter96_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter96_push(counter96_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter96_total(const counter96_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter96_dump(const counter96_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter96_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter96_free(counter96_t *c) {
    while (c != NULL) {
        counter96_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter97 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter97 *next;
};

typedef struct Counter97 counter97_t;

counter97_t *counter97_new(const char *name, enum Kind kind) {
    counter97_t *c = (counter97_t *)malloc(sizeof(counter97_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter97_push(counter97_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter97_total(const counter97_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter97_dump(const counter97_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter97_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter97_free(counter97_t *c) {
    while (c != NULL) {
        counter97_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter98 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter98 *next;
};

typedef struct Counter98 counter98_t;

counter98_t *counter98_new(const char *name, enum Kind kind) {
    counter98_t *c = (counter98_t *)malloc(sizeof(counter98_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter98_push(counter98_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter98_total(const counter98_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter98_dump(const counter98_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter98_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter98_free(counter98_t *c) {
    while (c != NULL) {
        counter98_t *next = c->next;
        free(c);
        c = next;
    }
}
struct Counter99 {
    const char *name;
    int values[MAX_VALUES];
    counter_size_t len;
    enum Kind kind;
    struct Counter99 *next;
};

typedef struct Counter99 counter99_t;

counter99_t *counter99_new(const char *name, enum Kind kind) {
    counter99_t *c = (counter99_t *)malloc(sizeof(counter99_t));
    if (c == NULL) return NULL;
    c->name = name;
    c->len = 0;
    c->kind = kind;
    c->next = NULL;
    return c;
}

int counter99_push(counter99_t *c, int v) {
    if (c == NULL) return -1;
    if (c->len >= MAX_VALUES) return -2;
    c->values[c->len++] = v;
    return 0;
}

int counter99_total(const counter99_t *c) {
    if (c == NULL) return 0;
    return sum_ints(c->values, c->len);
}

void counter99_dump(const counter99_t *c, FILE *out) {
    if (c == NULL || out == NULL) return;
    fprintf(out, "%s (n=%u, total=%d)\n", c->name, c->len, counter99_total(c));
    for (counter_size_t i = 0; i < c->len; i++) {
        fprintf(out, "  %d -> %s\n", c->values[i], classify(c->values[i]));
    }
}

void counter99_free(counter99_t *c) {
    while (c != NULL) {
        counter99_t *next = c->next;
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
