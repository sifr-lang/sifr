from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1254: Number Of Closed Islands
# Python version

def closedIsland(grid: List[List[int]]) -> int:
    r = len(grid)
    c = len(grid[0])
    seen = set()

    def dfs(x, y):
        if x < 0 or x >= r or y < 0 or y >= c or (x, y) in seen or grid[x][y] == 1:
            return
        seen.add((x, y))
        grid[x][y] = 1
        dfs(x+1, y)
        dfs(x, y+1)
        dfs(x-1, y)
        dfs(x, y-1)
    
    for i in range(r):
        for j in range(c):
            if i == 0 or j == 0 or i == r-1 or j == c-1:
                dfs(i, j)
    ans = 0
    for i in range(r):
        for j in range(c):
            if grid[i][j] == 0:
                dfs(i, j)
                ans += 1
    return ans       


def main():
    assert closedIsland([[1, 1, 1, 1, 1, 1, 1, 0], [1, 0, 0, 0, 0, 1, 1, 0], [1, 0, 1, 0, 1, 1, 1, 0], [1, 0, 0, 0, 0, 1, 0, 1], [1, 1, 1, 1, 1, 1, 1, 0]]) == 2
    assert closedIsland([[0, 0, 1, 0, 0], [0, 1, 0, 1, 0], [0, 1, 1, 1, 0]]) == 1
    assert closedIsland([[1, 1, 1, 1, 1, 1, 1], [1, 0, 0, 0, 0, 0, 1], [1, 0, 1, 1, 1, 0, 1], [1, 0, 1, 0, 1, 0, 1], [1, 0, 1, 1, 1, 0, 1], [1, 0, 0, 0, 0, 0, 1], [1, 1, 1, 1, 1, 1, 1]]) == 2

if __name__ == "__main__":
    main()
