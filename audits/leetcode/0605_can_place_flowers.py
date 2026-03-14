from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 605: Can Place Flowers
# Python version with test cases

def canPlaceFlowers(flowerbed: list[int], n: int) -> bool:
    count: int = 0
    for i in range(len(flowerbed)):
        if flowerbed[i] == 0:
            empty_left: bool = (i == 0) or (flowerbed[i - 1] == 0)
            empty_right: bool = (i == len(flowerbed) - 1) or (flowerbed[i + 1] == 0)
            if empty_left and empty_right:
                flowerbed[i] = 1
                count += 1
    return count >= n

def main():
    assert canPlaceFlowers([1, 0, 0, 0, 1], 1) == True
    assert canPlaceFlowers([1, 0, 0, 0, 1], 2) == False
    assert canPlaceFlowers([0, 0, 1, 0, 0], 1) == True
    assert canPlaceFlowers([0], 1) == True

if __name__ == "__main__":
    main()
