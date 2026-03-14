from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 680: Valid Palindrome Ii
# Python version

def validPalindrome(s: str) -> bool:
    i, j = 0, len(s) - 1
    
    while i < j:
        if s[i] == s[j]:
            i += 1
            j -= 1
        else:
            return validPalindromeUtil(s, i + 1, j) or validPalindromeUtil(s, i, j - 1)
    return True


def validPalindromeUtil(s, i, j):
    while i < j:
        if s[i] == s[j]:
            i += 1
            j -= 1
        else:
            return False
    
    return True



def main():
    assert validPalindrome("aba") == True
    assert validPalindrome("abca") == True
    assert validPalindrome("abc") == False

if __name__ == "__main__":
    main()
