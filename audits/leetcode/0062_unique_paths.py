from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 62: Unique Paths
# Python version

def uniquePaths(m: int, n: int) -> int:
    row = [1] * n

    for i in range(m - 1):
        newRow = [1] * n
        for j in range(n - 2, -1, -1):
            newRow[j] = newRow[j + 1] + row[j]
        row = newRow
    return row[0]

    # O(n * m) O(n)



def main():
    assert uniquePaths(3, 7) == 28
    assert uniquePaths(3, 2) == 3

if __name__ == "__main__":
    main()
