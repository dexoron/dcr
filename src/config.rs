// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub const PROFILE: &str = "debug";
pub const FILE_MAIN_C: &str = r#"#include <stdio.h>

int main(void) {
    printf("Hello World!\n");
    return 0;
}
"#;

pub const FILE_DCR_TEST_H: &str = r#"#pragma once
#include <stdio.h>
#include <string.h>

#define DCR_TEST_PASS 0
#define DCR_TEST_FAIL 1
#define DCR_TEST_SKIP 2

#define EXPECT(expr) \
    do { \
        if (!(expr)) { \
            fprintf(stderr, "assert failed: %s (%s:%d)\n", #expr, __FILE__, __LINE__); \
            return DCR_TEST_FAIL; \
        } \
    } while (0)

#define SKIP(reason) \
    do { \
        (void)(reason); \
        return DCR_TEST_SKIP; \
    } while (0)

#define TEST(name) static int name(void)
#define TEST_CASE(name) {#name, name}

typedef int (*test_fn_t)(void);

typedef struct {
    const char *name;
    test_fn_t fn;
} test_case_t;

static inline int dcr_test_raw_mode(int argc, char **argv) {
    return argc > 1 && strcmp(argv[1], "--dcr-raw") == 0;
}

static inline void dcr_test_print_result(const char *name, int rc, int raw_mode) {
    const char *tag = (rc == DCR_TEST_PASS) ? "PASS" : (rc == DCR_TEST_SKIP) ? "SKIP" : "FAIL";
    if (raw_mode) {
        printf("%s\t%s\n", tag, name);
    } else {
        printf("[%s] %s\n", tag, name);
    }
}
"#;

pub const FILE_TEST_C: &str = r#"#include "dcr_test.h"

TEST(test_example) {
    EXPECT(1 + 1 == 2);
    return DCR_TEST_PASS;
}

static test_case_t CASES[] = {
    TEST_CASE(test_example),
};

int main(int argc, char **argv) {
    int raw = dcr_test_raw_mode(argc, argv);
    int rc = 0;
    int count = (int)(sizeof(CASES) / sizeof(CASES[0]));
    for (int i = 0; i < count; ++i) {
        int status = CASES[i].fn();
        dcr_test_print_result(CASES[i].name, status, raw);
        if (status == DCR_TEST_FAIL) {
            rc = 1;
        }
    }
    return rc;
}
"#;

pub fn flags(profile: &str) -> Option<&'static [&'static str]> {
    let f = crate::utils::build::default_profile_flags(profile);
    if f.is_empty() { None } else { Some(f) }
}
