from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 946: Validate Stack Sequences
# Python version

def validateStackSequences(pushed, popped):
    i = 0
    stack = []
    for n in pushed:
        stack.append(n)
        while i < len(popped) and stack and popped[i] == stack[-1]:
            stack.pop()
            i += 1

    return not stack





def main():
    assert validateStackSequences([1,2,3,4,5], [4,5,3,2,1]) == True
    assert validateStackSequences([1,2,3,4,5], [4,3,5,1,2]) == False

if __name__ == "__main__":
    main()
