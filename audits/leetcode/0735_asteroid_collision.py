from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 735: Asteroid Collision
# Python version

def asteroidCollision(asteroids: List[int]) -> List[int]:
    stack = []

    for a in asteroids:
        while stack and a < 0 and stack[-1] > 0:
            diff = a + stack[-1]
            if diff > 0:
                a = 0
            elif diff < 0:
                stack.pop()
            else:
                a = 0
                stack.pop()
        if a:
            stack.append(a)

    return stack



def main():
    assert asteroidCollision([5,10,-5]) == [5, 10]
    assert asteroidCollision([8,-8]) == []
    assert asteroidCollision([10,2,-5]) == [10]

if __name__ == "__main__":
    main()
