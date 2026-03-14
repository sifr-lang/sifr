from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 678: Valid Parenthesis String
# Python version

def checkValidString(s: str) -> bool:
    dp = {(len(s), 0): True}  # key=(i, leftCount) -> isValid

    def dfs(i, left):
        if i == len(s) or left < 0:
            return left == 0
        if (i, left) in dp:
            return dp[(i, left)]

        if s[i] == "(":
            dp[(i, left)] = dfs(i + 1, left + 1)
        elif s[i] == ")":
            dp[(i, left)] = dfs(i + 1, left - 1)
        else:
            dp[(i, left)] = (
                dfs(i + 1, left + 1) or dfs(i + 1, left - 1) or dfs(i + 1, left)
            )
        return dp[(i, left)]

    return dfs(0, 0)


# Greedy: O(n)

def checkValidString(s: str) -> bool:
    leftMin, leftMax = 0, 0

    for c in s:
        if c == "(":
            leftMin, leftMax = leftMin + 1, leftMax + 1
        elif c == ")":
            leftMin, leftMax = leftMin - 1, leftMax - 1
        else:
            leftMin, leftMax = leftMin - 1, leftMax + 1
        if leftMax < 0:
            return False
        if leftMin < 0:  # required because -> s = ( * ) (
            leftMin = 0
    return leftMin == 0



def main():
    assert checkValidString("()") == True
    assert checkValidString("(*)") == True
    assert checkValidString("(*))") == True

if __name__ == "__main__":
    main()
