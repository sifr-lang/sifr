from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 261: Graph Valid Tree
# Python version

"""
@param n: An integer
@param edges: a list of undirected edges
@return: true if it's a valid tree, or false
"""


def validTree(n, edges):
    if not n:
        return True
    adj = {i: [] for i in range(n)}
    for n1, n2 in edges:
        adj[n1].append(n2)
        adj[n2].append(n1)

    visit = set()

    def dfs(i, prev):
        if i in visit:
            return False

        visit.add(i)
        for j in adj[i]:
            if j == prev:
                continue
            if not dfs(j, i):
                return False
        return True

    return dfs(0, -1) and n == len(visit)


# alternative solution via DSU. It avoids building an adjacency walk stack
# and instead rejects cycles while tracking connected components directly.
def validTreeDsu(n, edges):
    if n == 0:
        return True

    parents = [i for i in range(n)]
    ranks = [1] * n
    components = n

    def find(node):
        while node != parents[node]:
            parents[node] = parents[parents[node]]
            node = parents[node]
        return node

    def union(a, b):
        nonlocal components
        root_a = find(a)
        root_b = find(b)
        if root_a == root_b:
            return False
        if ranks[root_a] < ranks[root_b]:
            parents[root_a] = root_b
        elif ranks[root_a] > ranks[root_b]:
            parents[root_b] = root_a
        else:
            parents[root_b] = root_a
            ranks[root_a] += 1
        components -= 1
        return True

    for a, b in edges:
        if not union(a, b):
            return False

    return components == 1

def main():
    assert validTree(5, [[0, 1], [0, 2], [0, 3], [1, 4]]) is True
    assert validTree(5, [[0, 1], [1, 2], [2, 3], [1, 3], [1, 4]]) is False
    assert validTreeDsu(5, [[0, 1], [0, 2], [0, 3], [1, 4]]) is True
    assert validTreeDsu(5, [[0, 1], [1, 2], [2, 3], [1, 3], [1, 4]]) is False

if __name__ == "__main__":
    main()
