from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 63: Unique Paths Ii
# Python version

def uniquePathsWithObstacles(grid: List[List[int]]) -> int:
    M, N = len(grid), len(grid[0])
    dp = [0] * N
    dp[N-1] = 1

    # Time: O(N*M), Space: O(N)
    for r in reversed(range(M)):
        for c in reversed(range(N)):
            if grid[r][c]:
                dp[c] = 0
            elif c + 1 < N:
                dp[c] = dp[c] + dp[c + 1]
    return dp[0]


    # Time: O(N*M), Space: O(N*M)
    M, N = len(grid), len(grid[0])
    dp = {(M - 1, N - 1): 1}

    def dfs(r, c):
        if r == M or c == N or grid[r][c]:
            return 0
        if (r, c) in dp:
            return dp[(r, c)]
        dp[(r, c)] = dfs(r + 1, c) + dfs(r, c + 1)
        return dp[(r, c)]
    return dfs(0, 0)



def main():
    assert uniquePathsWithObstacles([[0,0,0],[0,1,0],[0,0,0]]) == 2

if __name__ == "__main__":
    main()
