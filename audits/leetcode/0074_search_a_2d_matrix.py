from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 74: Search A 2D Matrix
# Python version

def searchMatrix(matrix: List[List[int]], target: int) -> bool:
    ROWS, COLS = len(matrix), len(matrix[0])

    top, bot = 0, ROWS - 1
    while top <= bot:
        row = (top + bot) // 2
        if target > matrix[row][-1]:
            top = row + 1
        elif target < matrix[row][0]:
            bot = row - 1
        else:
            break

    if not (top <= bot):
        return False
    row = (top + bot) // 2
    l, r = 0, COLS - 1
    while l <= r:
        m = (l + r) // 2
        if target > matrix[row][m]:
            l = m + 1
        elif target < matrix[row][m]:
            r = m - 1
        else:
            return True
    return False



def main():
    assert searchMatrix([[1,3,5,7],[10,11,16,20],[23,30,34,60]], 3) == True
    assert searchMatrix([[1,3,5,7],[10,11,16,20],[23,30,34,60]], 13) == False

if __name__ == "__main__":
    main()
