from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 71: Simplify Path
# Python version

def simplifyPath(path: str) -> str:

    stack = []

    for i in path.split("/"):
        #  if i == "/" or i == '//', it becomes '' (empty string)

        if i == "..":
            if stack:
                stack.pop()
        elif i == "." or i == '':
            # skip "." or an empty string
            continue
        else:
            stack.append(i)

    res = "/" + "/".join(stack)
    return res



def main():
    assert simplifyPath('/home/') == '/home'
    assert simplifyPath('/home//foo/') == '/home/foo'
    assert simplifyPath('/home/user/Documents/../Pictures') == '/home/user/Pictures'

if __name__ == "__main__":
    main()
