from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2101: Detonate The Maximum Bombs
# Python version

def maximumDetonation(bombs: List[List[int]]) -> int:
    n = len(bombs)
    graph = [[] for _ in range(n)]

    for i in range(n):
        for j in range(n):
            if i != j:
                x1, y1, r1 = bombs[i]
                x2, y2, _ = bombs[j]

                dst = sqrt((x1 - x2) ** 2 + (y1 - y2) ** 2)

                if dst <= r1:
                    graph[i].append(j)

    def dfs(node, vis):
        vis[node] = True
        count = 1

        for nbh in graph[node]:
            if not vis[nbh]:
                count += dfs(nbh, vis)
                
        return count

    detonated = 0

    for i in range(n):
        visited = [False] * n
        detonated = max(detonated, dfs(i, visited))
    
    return detonated


def main():
    assert maximumDetonation([[2, 1, 3], [6, 1, 4]]) == 2
    assert maximumDetonation([[1, 1, 5], [10, 10, 5]]) == 1
    assert maximumDetonation([[1, 2, 3], [2, 3, 1], [3, 4, 2], [4, 5, 3], [5, 6, 4]]) == 5

if __name__ == "__main__":
    main()
