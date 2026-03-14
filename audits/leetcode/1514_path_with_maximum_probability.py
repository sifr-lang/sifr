from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1514: Path With Maximum Probability
# Python version

def maxProbability(n: int, edges: List[List[int]], succProb: List[float], start: int, end: int) -> float:
    adj = collections.defaultdict(list)
    for i in range(len(edges)):
        src, dst = edges[i]
        adj[src].append([dst, succProb[i]])
        adj[dst].append([src, succProb[i]])

    pq = [(-1, start)]
    visit = set()

    while pq:
        prob, cur = heapq.heappop(pq)
        visit.add(cur)

        if cur == end:
            return prob * -1
        for nei, edgeProb in adj[cur]:
            if nei not in visit:
                heapq.heappush(pq, (prob * edgeProb, nei))
    return 0



def main():
    assert maxProbability(3, [[0, 1], [1, 2], [0, 2]], [0.5, 0.5, 0.2], 0, 2) == 0.25
    assert maxProbability(3, [[0, 1], [1, 2], [0, 2]], [0.5, 0.5, 0.3], 0, 2) == 0.3
    assert maxProbability(3, [[0, 1]], [0.5], 0, 2) == 0

if __name__ == "__main__":
    main()
