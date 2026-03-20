/*
 * strlen.c — Compute the length of a null-terminated string.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"

/*@
  requires valid_string(s);
  assigns \nothing;
  ensures \result == strlen(s);
*/
size_t gen_strlen(const char *s)
{
    size_t len = 0;

    /*@
      loop invariant 0 <= len;
      loop assigns len;
      loop variant strlen(s) - len;
    */
    while (s[len] != '\0') {
        len++;
    }

    return len;
}
