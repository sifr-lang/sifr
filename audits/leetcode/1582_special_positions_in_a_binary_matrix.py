from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1582: Special Positions In A Binary Matrix
# Python version

def numSpecial(mat: List[List[int]]) -> int:
    m = len(mat)
    n = len(mat[0])
    rowCount = [0] * m
    colCount = [0] * n
    res = 0
    for r in range(m):
        for c in range(n):
            if mat[r][c] == 1:
                rowCount[r] += 1
                colCount[c] += 1
    for r in range(m):
        for c in range(n):
            if mat[r][c] == 1 and rowCount[r] == 1 and colCount[c] == 1:
                res += 1
    return res
    



def main():
    assert numSpecial([[1, 0, 0], [0, 0, 1], [1, 0, 0]]) == 1
    assert numSpecial([[1, 0, 0], [0, 1, 0], [0, 0, 1]]) == 3

if __name__ == "__main__":
    main()
