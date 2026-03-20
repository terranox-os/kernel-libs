/*
 * strncmp.c — Compare at most n bytes of two strings.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"

/*@
  requires n == 0 || \valid_read(s1 + (0 .. n - 1));
  requires n == 0 || \valid_read(s2 + (0 .. n - 1));
  assigns \nothing;
  ensures n == 0 ==> \result == 0;
*/
int gen_strncmp(const char *s1, const char *s2, size_t n)
{
    /*@
      loop invariant 0 <= n;
      loop assigns n, s1, s2;
      loop variant n;
    */
    while (n > 0) {
        unsigned char c1 = (unsigned char)*s1;
        unsigned char c2 = (unsigned char)*s2;

        if (c1 != c2) {
            return (c1 < c2) ? -1 : 1;
        }
        if (c1 == 0) {
            return 0;
        }

        s1++;
        s2++;
        n--;
    }

    return 0;
}
