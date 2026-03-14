from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2002: Maximum Product Of The Length Of Two Palindromic Subsequences
# Python version

def maxProduct(s):
    n = len(s)
    
    first, last = [0]*(1<<n), [0]*(1<<n)
    
    for i in range(n):
        for j in range(1<<i, 1<<(i+1)):
            first[j] = i

    for i in range(n):
        for j in range(1<<i, 1<<n, 1<<(i+1)):
            last[j] = i
    
    @lru_cache(None)
    def dp(m):
        if m & (m-1) == 0: return m != 0
        l, f = last[m], first[m]
        lb, fb = 1<<l, 1<<f
        return max(dp(m-lb), dp(m-fb), dp(m-lb-fb) + (s[l] == s[f]) * 2)
   
    ans = 0
    for m in range(1, 1<<n):
        ans = max(ans, dp(m)*dp((1<<n) - 1 - m))
        
    return ans

"""
Time Complexity: O(2^N)
Space Complexity: O(2^N)
"""

def main():
    assert maxProduct('leetcodecom') == 9
    assert maxProduct('bb') == 1
    assert maxProduct('accbcaxxcxx') == 25

if __name__ == "__main__":
    main()
