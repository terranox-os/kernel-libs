/*
 * strlen.c — Compute the length of a null-terminated string.
 *
 * Freestanding. ACSL-annotated for Frama-C WP verification.
 */

#include "gen_primitives.h"

/*@
  requires \exists integer n; n >= 0 && \valid_read(s + (0 .. n)) && s[n] == '\0';
  assigns \nothing;
  ensures s[\result] == '\0';
  ensures \forall integer k; 0 <= k < \result ==> s[k] != '\0';
*/
size_t gen_strlen(const char *s)
{
    size_t len = 0;

    /*@
      loop invariant 0 <= len;
      loop invariant \forall integer k; 0 <= k < len ==> s[k] != '\0';
      loop assigns len;
    */
    while (s[len] != '\0') {
        len++;
    }

    return len;
}
