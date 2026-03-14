from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2092: Find All People With Secret
# Python version

def findAllPeople(n: int, meetings: List[List[int]], firstPerson: int) -> List[int]:
    secrets = set([0, firstPerson])
    time_map = {}

    for src, dst, t in meetings:
        if t not in time_map:
            time_map[t] = defaultdict(list)
        time_map[t][src].append(dst)
        time_map[t][dst].append(src)
    
    def dfs(src, adj):
        if src in visit:
            return
        visit.add(src)
        secrets.add(src)
        for nei in adj[src]:
            dfs(nei, adj)

    for t in sorted(time_map.keys()):
        visit = set()
        for src in time_map[t]:
            if src in secrets:
                dfs(src, time_map[t])
    return list(secrets)



def main():
    assert findAllPeople(6, [[1, 2, 5], [2, 3, 8], [1, 5, 10]], 1) == [0, 1, 2, 3, 5]
    assert findAllPeople(4, [[3, 1, 3], [1, 2, 2], [0, 3, 3]], 3) == [0, 1, 3]
    assert findAllPeople(5, [[3, 4, 2], [1, 2, 1], [2, 3, 1]], 1) == [0, 1, 2, 3, 4]

if __name__ == "__main__":
    main()
