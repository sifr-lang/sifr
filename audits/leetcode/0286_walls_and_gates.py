from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 286: Walls And Gates
# Python version

"""
@param rooms: m x n 2D grid
@return: nothing
"""


def walls_and_gates(rooms: List[List[int]]):
    ROWS, COLS = len(rooms), len(rooms[0])
    visit = set()
    q = deque()

    def addRooms(r, c):
        if (
            min(r, c) < 0
            or r == ROWS
            or c == COLS
            or (r, c) in visit
            or rooms[r][c] == -1
        ):
            return
        visit.add((r, c))
        q.append([r, c])

    for r in range(ROWS):
        for c in range(COLS):
            if rooms[r][c] == 0:
                q.append([r, c])
                visit.add((r, c))

    dist = 0
    while q:
        for i in range(len(q)):
            r, c = q.popleft()
            rooms[r][c] = dist
            addRooms(r + 1, c)
            addRooms(r - 1, c)
            addRooms(r, c + 1)
            addRooms(r, c - 1)
        dist += 1



def main():
    arg0 = [[2147483647, -1, 0, 2147483647], [2147483647, 2147483647, 2147483647, -1], [2147483647, -1, 2147483647, -1], [0, -1, 2147483647, 2147483647]]
    _result = walls_and_gates(arg0)
    assert arg0 == [[3, -1, 0, 1], [2, 2, 1, -1], [1, -1, 2, -1], [0, -1, 3, 4]]
    arg0 = [[-1]]
    _result = walls_and_gates(arg0)
    assert arg0 == [[-1]]

if __name__ == "__main__":
    main()
