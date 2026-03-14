from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1209: Remove All Adjacent Duplicates In String Ii
# Python version

def removeDuplicates(s: str, k: int) -> str:
    stack = []  # [char, count]

    for c in s:
        if stack and stack[-1][0] == c:
            stack[-1][1] += 1
        else:
            stack.append([c, 1])

        if stack[-1][1] == k:
            stack.pop()

    res = ""
    for char, count in stack:
        res += char * count

    return res



def main():
    assert removeDuplicates("abcd", 2) == 'abcd'
    assert removeDuplicates("deeedbbcccbdaa", 3) == 'aa'

if __name__ == "__main__":
    main()
