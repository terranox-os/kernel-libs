/*
 * kprintf.c — Callback-based printf engine.
 *
 * Freestanding: no libc, no heap, no global state.
 * All output goes through the caller-provided gen_output_fn.
 */

#include "gen_kfmt.h"

/* ── Internal helpers ────────────────────────────────────── */

static void emit(gen_output_fn out, void *ctx, uint8_t byte, int32_t *count)
{
    out(byte, ctx);
    (*count)++;
}

static void emit_str(gen_output_fn out, void *ctx, const char *s, int32_t *count)
{
    while (*s) {
        emit(out, ctx, (uint8_t)*s++, count);
    }
}

static void emit_pad(gen_output_fn out, void *ctx, uint8_t ch,
                     int32_t width, int32_t len, int32_t *count)
{
    while (len < width) {
        emit(out, ctx, ch, count);
        len++;
    }
}

/*
 * Render an unsigned 64-bit value in the given base.
 * buf must have room for at least 20 characters (max uint64 decimal digits).
 * Returns the number of characters written to buf.
 */
static int32_t render_unsigned(char *buf, uint64_t val, int base, int upper)
{
    const char *digits = upper ? "0123456789ABCDEF" : "0123456789abcdef";
    int32_t len = 0;

    if (val == 0) {
        buf[0] = '0';
        return 1;
    }

    /* Render in reverse */
    while (val > 0) {
        buf[len++] = digits[val % (uint64_t)base];
        val /= (uint64_t)base;
    }

    /* Reverse in place */
    for (int32_t i = 0; i < len / 2; i++) {
        char tmp = buf[i];
        buf[i] = buf[len - 1 - i];
        buf[len - 1 - i] = tmp;
    }

    return len;
}

/* ── Public API ──────────────────────────────────────────── */

int32_t gen_kvprintf(gen_output_fn out, void *ctx, const char *fmt, va_list args)
{
    int32_t count = 0;
    char numbuf[24]; /* Enough for max uint64 in decimal + sign */

    while (*fmt) {
        if (*fmt != '%') {
            emit(out, ctx, (uint8_t)*fmt++, &count);
            continue;
        }

        fmt++; /* skip '%' */

        /* Parse flags */
        uint8_t pad_char = ' ';
        if (*fmt == '0') {
            pad_char = '0';
            fmt++;
        }

        /* Parse width */
        int32_t width = 0;
        while (*fmt >= '0' && *fmt <= '9') {
            width = width * 10 + (*fmt - '0');
            fmt++;
        }

        /* Parse 'l' length modifier for 64-bit */
        int is_long = 0;
        if (*fmt == 'l') {
            is_long = 1;
            fmt++;
            if (*fmt == 'l') {
                fmt++; /* 'll' — still 64-bit */
            }
        }

        /* Dispatch on specifier */
        switch (*fmt) {
        case 'd':
        case 'i': {
            int64_t val;
            if (is_long) {
                val = va_arg(args, int64_t);
            } else {
                val = va_arg(args, int32_t);
            }

            int negative = 0;
            uint64_t uval;
            if (val < 0) {
                negative = 1;
                uval = (uint64_t)(-(val + 1)) + 1;
            } else {
                uval = (uint64_t)val;
            }

            int32_t len = render_unsigned(numbuf, uval, 10, 0);

            int32_t total_len = len + negative;
            if (pad_char == '0' && negative) {
                emit(out, ctx, '-', &count);
                emit_pad(out, ctx, '0', width, total_len, &count);
            } else {
                if (pad_char == ' ') {
                    emit_pad(out, ctx, ' ', width, total_len, &count);
                }
                if (negative) {
                    emit(out, ctx, '-', &count);
                }
                if (pad_char == '0') {
                    emit_pad(out, ctx, '0', width, total_len, &count);
                }
            }

            for (int32_t i = 0; i < len; i++) {
                emit(out, ctx, (uint8_t)numbuf[i], &count);
            }
            break;
        }

        case 'u': {
            uint64_t val;
            if (is_long) {
                val = va_arg(args, uint64_t);
            } else {
                val = va_arg(args, uint32_t);
            }
            int32_t len = render_unsigned(numbuf, val, 10, 0);
            emit_pad(out, ctx, pad_char, width, len, &count);
            for (int32_t i = 0; i < len; i++) {
                emit(out, ctx, (uint8_t)numbuf[i], &count);
            }
            break;
        }

        case 'x':
        case 'X': {
            uint64_t val;
            if (is_long) {
                val = va_arg(args, uint64_t);
            } else {
                val = va_arg(args, uint32_t);
            }
            int upper = (*fmt == 'X');
            int32_t len = render_unsigned(numbuf, val, 16, upper);
            emit_pad(out, ctx, pad_char, width, len, &count);
            for (int32_t i = 0; i < len; i++) {
                emit(out, ctx, (uint8_t)numbuf[i], &count);
            }
            break;
        }

        case 'p': {
            uintptr_t val = (uintptr_t)va_arg(args, void *);
            emit(out, ctx, '0', &count);
            emit(out, ctx, 'x', &count);
            int32_t len = render_unsigned(numbuf, (uint64_t)val, 16, 0);
            emit_pad(out, ctx, '0', width > 2 ? width - 2 : 0, len, &count);
            for (int32_t i = 0; i < len; i++) {
                emit(out, ctx, (uint8_t)numbuf[i], &count);
            }
            break;
        }

        case 's': {
            const char *s = va_arg(args, const char *);
            if (!s) {
                s = "(null)";
            }
            /* Compute length for padding */
            int32_t slen = 0;
            const char *p = s;
            while (*p++) slen++;

            emit_pad(out, ctx, ' ', width, slen, &count);
            emit_str(out, ctx, s, &count);
            break;
        }

        case 'c': {
            uint8_t ch = (uint8_t)va_arg(args, int);
            emit_pad(out, ctx, ' ', width, 1, &count);
            emit(out, ctx, ch, &count);
            break;
        }

        case '%':
            emit(out, ctx, '%', &count);
            break;

        case '\0':
            /* Trailing '%' at end of string */
            goto done;

        default:
            /* Unknown specifier: emit literally */
            emit(out, ctx, '%', &count);
            emit(out, ctx, (uint8_t)*fmt, &count);
            break;
        }

        fmt++;
    }

done:
    return count;
}

int32_t gen_kprintf(gen_output_fn out, void *ctx, const char *fmt, ...)
{
    va_list args;
    va_start(args, fmt);
    int32_t count = gen_kvprintf(out, ctx, fmt, args);
    va_end(args);
    return count;
}
