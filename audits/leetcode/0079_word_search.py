from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 79: Word Search
# Python version

def exist(board: List[List[str]], word: str) -> bool:
    ROWS, COLS = len(board), len(board[0])
    path = set()

    def dfs(r, c, i):
        if i == len(word):
            return True
        if (
            min(r, c) < 0
            or r >= ROWS
            or c >= COLS
            or word[i] != board[r][c]
            or (r, c) in path
        ):
            return False
        path.add((r, c))
        res = (
            dfs(r + 1, c, i + 1)
            or dfs(r - 1, c, i + 1)
            or dfs(r, c + 1, i + 1)
            or dfs(r, c - 1, i + 1)
        )
        path.remove((r, c))
        return res

    # To prevent TLE,reverse the word if frequency of the first letter is more than the last letter's
    count = sum(map(Counter, board), Counter())
    if count[word[0]] > count[word[-1]]:
        word = word[::-1]
        
    for r in range(ROWS):
        for c in range(COLS):
            if dfs(r, c, 0):
                return True
    return False

# O(n * m * 4^n)



def main():
    assert exist([['A', 'B', 'C', 'E'], ['S', 'F', 'C', 'S'], ['A', 'D', 'E', 'E']], 'ABCCED') == True
    assert exist([['A', 'B', 'C', 'E'], ['S', 'F', 'C', 'S'], ['A', 'D', 'E', 'E']], 'SEE') == True
    assert exist([['A', 'B', 'C', 'E'], ['S', 'F', 'C', 'S'], ['A', 'D', 'E', 'E']], 'ABCB') == False

if __name__ == "__main__":
    main()
