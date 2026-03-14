from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 695: Max Area Of Island
# Python version

def maxAreaOfIsland(grid: List[List[int]]) -> int:
    ROWS, COLS = len(grid), len(grid[0])
    visit = set()

    def dfs(r, c):
        if (
            r < 0
            or r == ROWS
            or c < 0
            or c == COLS
            or grid[r][c] == 0
            or (r, c) in visit
        ):
            return 0
        visit.add((r, c))
        return 1 + dfs(r + 1, c) + dfs(r - 1, c) + dfs(r, c + 1) + dfs(r, c - 1)

    area = 0
    for r in range(ROWS):
        for c in range(COLS):
            area = max(area, dfs(r, c))
    return area



def main():
    assert maxAreaOfIsland([[0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0], [0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0], [0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0], [0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0]]) == 6
    assert maxAreaOfIsland([[0, 0, 0, 0, 0, 0, 0, 0]]) == 0

if __name__ == "__main__":
    main()
