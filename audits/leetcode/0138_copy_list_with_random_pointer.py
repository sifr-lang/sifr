from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 138: Copy List With Random Pointer
# Python version

class Node:
    def __init__(self, x: int, next: 'Node | None' = None, random: 'Node | None' = None):
        self.val = int(x)
        self.next = next
        self.random = random

def build_random_list(spec: list[tuple[int, int]]) -> Node | None:
    if len(spec) == 0:
        return None
    nodes = [Node(val) for val, _ in spec]
    for i in range(len(nodes) - 1):
        nodes[i].next = nodes[i + 1]
    for i, (_, random_index) in enumerate(spec):
        if random_index >= 0:
            nodes[i].random = nodes[random_index]
    return nodes[0]


def random_list_to_pairs(head: Node | None) -> list[tuple[int, int]]:
    nodes = []
    cur = head
    while cur is not None:
        nodes.append(cur)
        cur = cur.next
    indices = {node: i for i, node in enumerate(nodes)}
    pairs = []
    for node in nodes:
        random_index = -1 if node.random is None else indices[node.random]
        pairs.append((node.val, random_index))
    return pairs

def copyRandomList(head: "Node") -> "Node":
    oldToCopy = {None: None}

    cur = head
    while cur:
        copy = Node(cur.val)
        oldToCopy[cur] = copy
        cur = cur.next
    cur = head
    while cur:
        copy = oldToCopy[cur]
        copy.next = oldToCopy[cur.next]
        copy.random = oldToCopy[cur.random]
        cur = cur.next
    return oldToCopy[head]


def main():
    assert random_list_to_pairs(copyRandomList(build_random_list([(7, -1), (13, 0), (11, 4), (10, 2), (1, 0)]))) == [(7, -1), (13, 0), (11, 4), (10, 2), (1, 0)]
    assert random_list_to_pairs(copyRandomList(build_random_list([(1, 1), (2, 1)]))) == [(1, 1), (2, 1)]
    assert random_list_to_pairs(copyRandomList(build_random_list([(3, -1), (3, 0), (3, -1)]))) == [(3, -1), (3, 0), (3, -1)]

if __name__ == "__main__":
    main()
