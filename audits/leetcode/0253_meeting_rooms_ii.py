from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 253: Meeting Rooms Ii
# Python version

def minMeetingRooms(self, intervals: List[List[int]]) -> int:
        time = []
        for start, end in intervals:
            time.append((start, 1))
            time.append((end, -1))
        time.sort(key=lambda x: (x[0], x[1]))
        count = 0
        max_count = 0
        for t in time:
            count += t[1]
            max_count = max(max_count, count)
        return max_count

def main():
    assert minMeetingRooms(None, [[0, 30], [5, 10], [15, 20]]) == 2
    assert minMeetingRooms(None, [[7, 10], [2, 4]]) == 1

if __name__ == "__main__":
    main()
