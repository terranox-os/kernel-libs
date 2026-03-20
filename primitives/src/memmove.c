/*
 * memmove.c — Memory copy that handles overlapping regions.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"
#include <stdint.h>

/*@
  requires n == 0 || \valid((unsigned char *)dst + (0 .. n - 1));
  requires n == 0 || \valid_read((unsigned char *)src + (0 .. n - 1));
  assigns ((unsigned char *)dst)[0 .. n - 1];
  ensures \result == dst;
*/
void *gen_memmove(void *dst, const void *src, size_t n)
{
    unsigned char       *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;

    if (d == s || n == 0) {
        return dst;
    }

    if (d < s || d >= s + n) {
        /* No overlap or dst is before src: forward copy */
        /*@
          loop invariant 0 <= n;
          loop assigns n, *d, d, s;
          loop variant n;
        */
        while (n > 0) {
            *d++ = *s++;
            n--;
        }
    } else {
        /* dst overlaps into src region: backward copy */
        d += n;
        s += n;
        /*@
          loop invariant 0 <= n;
          loop assigns n, d, s, *d;
          loop variant n;
        */
        while (n > 0) {
            *--d = *--s;
            n--;
        }
    }

    return dst;
}
