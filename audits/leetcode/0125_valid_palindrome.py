from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 125: Valid Palindrome
# Python version

def isPalindrome(s: str) -> bool:
    new = ''
    for a in s:
        if a.isalpha() or a.isdigit():
            new += a.lower()
    return (new == new[::-1])



def main():
    assert isPalindrome("A man, a plan, a canal: Panama") == True
    assert isPalindrome("race a car") == False

if __name__ == "__main__":
    main()
