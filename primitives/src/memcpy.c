/*
 * memcpy.c — Non-overlapping memory copy.
 *
 * Freestanding, no dependencies beyond stddef.h / stdint.h.
 * ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"
#include <stdint.h>

/*@
  requires n == 0 || \valid((unsigned char *)dst + (0 .. n - 1));
  requires n == 0 || \valid_read((unsigned char *)src + (0 .. n - 1));
  requires \separated((unsigned char *)dst + (0 .. n - 1),
                       (unsigned char *)src + (0 .. n - 1));
  assigns ((unsigned char *)dst)[0 .. n - 1];
  ensures \forall integer i; 0 <= i < n ==>
          ((unsigned char *)dst)[i] == ((unsigned char *)src)[i];
  ensures \result == dst;
*/
void *gen_memcpy(void *dst, const void *src, size_t n)
{
    unsigned char       *d = (unsigned char *)dst;
    const unsigned char *s = (const unsigned char *)src;

    /* Word-sized copy when both pointers are aligned */
    if (n >= sizeof(unsigned long) &&
        ((uintptr_t)d % sizeof(unsigned long) == 0) &&
        ((uintptr_t)s % sizeof(unsigned long) == 0))
    {
        unsigned long       *dw = (unsigned long *)d;
        const unsigned long *sw = (const unsigned long *)s;
        size_t words = n / sizeof(unsigned long);

        /*@
          loop invariant 0 <= words;
          loop assigns words, dw[0 ..], sw, dw;
          loop variant words;
        */
        while (words > 0) {
            *dw++ = *sw++;
            words--;
        }

        d = (unsigned char *)dw;
        s = (const unsigned char *)sw;
        n %= sizeof(unsigned long);
    }

    /*@
      loop invariant 0 <= n;
      loop assigns n, d[0 ..], d, s;
      loop variant n;
    */
    while (n > 0) {
        *d++ = *s++;
        n--;
    }

    return dst;
}
