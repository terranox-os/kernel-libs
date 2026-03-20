/*
 * strnlen.c — Bounded string length.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"

/*@
  requires maxlen == 0 || \valid_read(s + (0 .. maxlen - 1));
  assigns \nothing;
  ensures \result <= maxlen;
*/
size_t gen_strnlen(const char *s, size_t maxlen)
{
    size_t len = 0;

    /*@
      loop invariant 0 <= len <= maxlen;
      loop assigns len;
      loop variant maxlen - len;
    */
    while (len < maxlen && s[len] != '\0') {
        len++;
    }

    return len;
}
