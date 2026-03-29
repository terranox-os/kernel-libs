/*
 * intrusive_list.c --- Circular doubly-linked list with sentinel head.
 *
 * Mirrors the Rust IntrusiveList implementation. The list is circular:
 * head.next/prev point to the first/last real node, and an empty list
 * has head pointing to itself in both directions.
 *
 * Freestanding: no libc, no heap.
 */

#include "gen_collections.h"

void gen_list_init(GenList *list)
{
    list->head.next = &list->head;
    list->head.prev = &list->head;
}

int gen_list_is_empty(const GenList *list)
{
    return list->head.next == &list->head;
}

void gen_list_push_front(GenList *list, GenListNode *node)
{
    GenListNode *head = &list->head;

    node->next = head->next;
    node->prev = head;
    head->next->prev = node;
    head->next = node;
}

void gen_list_push_back(GenList *list, GenListNode *node)
{
    GenListNode *head = &list->head;

    node->next = head;
    node->prev = head->prev;
    head->prev->next = node;
    head->prev = node;
}

GenListNode *gen_list_pop_front(GenList *list)
{
    if (gen_list_is_empty(list))
        return NULL;

    GenListNode *node = list->head.next;
    gen_list_remove(node);
    return node;
}

GenListNode *gen_list_pop_back(GenList *list)
{
    if (gen_list_is_empty(list))
        return NULL;

    GenListNode *node = list->head.prev;
    gen_list_remove(node);
    return node;
}

void gen_list_remove(GenListNode *node)
{
    node->prev->next = node->next;
    node->next->prev = node->prev;
    /* Mark as unlinked: point to self. */
    node->next = node;
    node->prev = node;
}
