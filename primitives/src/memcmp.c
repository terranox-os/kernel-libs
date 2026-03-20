/*
 * memcmp.c — Compare two memory regions byte-by-byte.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"

/*@
  requires n == 0 || \valid_read((unsigned char *)s1 + (0 .. n - 1));
  requires n == 0 || \valid_read((unsigned char *)s2 + (0 .. n - 1));
  assigns \nothing;
  ensures n == 0 ==> \result == 0;
*/
int gen_memcmp(const void *s1, const void *s2, size_t n)
{
    const unsigned char *p1 = (const unsigned char *)s1;
    const unsigned char *p2 = (const unsigned char *)s2;

    /*@
      loop invariant 0 <= n;
      loop assigns n, p1, p2;
      loop variant n;
    */
    while (n > 0) {
        if (*p1 != *p2) {
            return (*p1 < *p2) ? -1 : 1;
        }
        p1++;
        p2++;
        n--;
    }

    return 0;
}
