/*
 * kprintf_test.c — Host-compiled tests for gen_kprintf.
 *
 * Uses a buffer-backed output callback to capture formatted output,
 * then compares against expected strings.
 */

#include "gen_kfmt.h"
#include <string.h>

static int failures = 0;

#define ASSERT(cond, msg)                                        \
    do {                                                         \
        if (!(cond)) {                                           \
            failures++;                                          \
        }                                                        \
    } while (0)

/* ── Test output backend ───────────────────────────────────── */

typedef struct {
    uint8_t buf[512];
    size_t  pos;
} TestBuf;

static void test_output(uint8_t byte, void *ctx)
{
    TestBuf *tb = (TestBuf *)ctx;
    if (tb->pos < sizeof(tb->buf) - 1) {
        tb->buf[tb->pos++] = byte;
    }
    tb->buf[tb->pos] = '\0';
}

static void reset(TestBuf *tb)
{
    tb->pos = 0;
    tb->buf[0] = '\0';
}

/* ── Helper ────────────────────────────────────────────────── */

static int streq(const TestBuf *tb, const char *expected)
{
    return strcmp((const char *)tb->buf, expected) == 0;
}

/* ── Tests ─────────────────────────────────────────────────── */

static void test_plain_string(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "hello world");
    ASSERT(streq(&tb, "hello world"), "plain string");
}

static void test_empty_string(void)
{
    TestBuf tb; reset(&tb);
    int32_t n = gen_kprintf(test_output, &tb, "");
    ASSERT(n == 0, "empty string count");
    ASSERT(tb.pos == 0, "empty string pos");
}

static void test_percent_literal(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "100%%");
    ASSERT(streq(&tb, "100%"), "percent literal");
}

static void test_decimal_positive(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%d", 42);
    ASSERT(streq(&tb, "42"), "decimal positive");
}

static void test_decimal_negative(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%d", -123);
    ASSERT(streq(&tb, "-123"), "decimal negative");
}

static void test_decimal_zero(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%d", 0);
    ASSERT(streq(&tb, "0"), "decimal zero");
}

static void test_unsigned(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%u", (uint32_t)4294967295U);
    ASSERT(streq(&tb, "4294967295"), "unsigned max");
}

static void test_hex_lower(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%x", (uint32_t)0xDEAD);
    ASSERT(streq(&tb, "dead"), "hex lower");
}

static void test_hex_upper(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%X", (uint32_t)0xBEEF);
    ASSERT(streq(&tb, "BEEF"), "hex upper");
}

static void test_hex_zero_padded(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%08x", (uint32_t)0xAB);
    ASSERT(streq(&tb, "000000ab"), "hex zero-padded");
}

static void test_string(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "hello %s", "world");
    ASSERT(streq(&tb, "hello world"), "string");
}

static void test_string_null(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%s", (const char *)0);
    ASSERT(streq(&tb, "(null)"), "null string");
}

static void test_char(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%c", 'A');
    ASSERT(streq(&tb, "A"), "char");
}

static void test_pointer(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%p", (void *)0x1000);
    ASSERT(streq(&tb, "0x1000"), "pointer");
}

static void test_pointer_null(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%p", (void *)0);
    ASSERT(streq(&tb, "0x0"), "null pointer");
}

static void test_width_decimal(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%5d", 42);
    ASSERT(streq(&tb, "   42"), "width decimal");
}

static void test_zero_pad_decimal(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%05d", 42);
    ASSERT(streq(&tb, "00042"), "zero-pad decimal");
}

static void test_zero_pad_negative(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%06d", -42);
    ASSERT(streq(&tb, "-00042"), "zero-pad negative");
}

static void test_mixed_format(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "pid=%d addr=%08x name=%s",
                1234, (uint32_t)0xFF00, "init");
    ASSERT(streq(&tb, "pid=1234 addr=0000ff00 name=init"), "mixed format");
}

static void test_return_count(void)
{
    TestBuf tb; reset(&tb);
    int32_t n = gen_kprintf(test_output, &tb, "abc%d", 42);
    ASSERT(n == 5, "return count");
}

static void test_string_width(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%10s", "hi");
    ASSERT(streq(&tb, "        hi"), "string width");
}

static void test_unknown_specifier(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%q");
    ASSERT(streq(&tb, "%q"), "unknown specifier passthrough");
}

static void test_int_min(void)
{
    TestBuf tb; reset(&tb);
    gen_kprintf(test_output, &tb, "%d", (int32_t)-2147483648);
    ASSERT(streq(&tb, "-2147483648"), "INT32_MIN");
}

/* ── Main ──────────────────────────────────────────────────── */

int main(void)
{
    test_plain_string();
    test_empty_string();
    test_percent_literal();
    test_decimal_positive();
    test_decimal_negative();
    test_decimal_zero();
    test_unsigned();
    test_hex_lower();
    test_hex_upper();
    test_hex_zero_padded();
    test_string();
    test_string_null();
    test_char();
    test_pointer();
    test_pointer_null();
    test_width_decimal();
    test_zero_pad_decimal();
    test_zero_pad_negative();
    test_mixed_format();
    test_return_count();
    test_string_width();
    test_unknown_specifier();
    test_int_min();

    return failures;
}
