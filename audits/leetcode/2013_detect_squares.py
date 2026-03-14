from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2013: Detect Squares
# Python version

class DetectSquares:
    def __init__(self):
        self.ptsCount = defaultdict(int)
        self.pts = []
    def add(self, point: List[int]) -> None:
        self.ptsCount[tuple(point)] += 1
        self.pts.append(point)
    def count(self, point: List[int]) -> int:
        res = 0
        px, py = point
        for x, y in self.pts:
            if (abs(py - y) != abs(px - x)) or x == px or y == py:
                continue
            res += self.ptsCount[(x, py)] * self.ptsCount[(px, y)]
        return res

def main():
    obj = DetectSquares()
    obj.add([3, 10])
    obj.add([11, 2])
    obj.add([3, 2])
    assert obj.count([11, 10]) == 1
    assert obj.count([14, 8]) == 0
    obj.add([11, 2])
    assert obj.count([11, 10]) == 2

if __name__ == "__main__":
    main()
