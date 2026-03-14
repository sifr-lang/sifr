from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 252: Meeting Rooms
# Python version

"""
@param intervals: an array of meeting time intervals
@return: if a person could attend all meetings
"""


def canAttendMeetings(intervals):
    intervals.sort(key=lambda i: i[0])

    for i in range(1, len(intervals)):
        i1 = intervals[i - 1]
        i2 = intervals[i]

        if i1[1] > i2[0]:
            return False
    return True



def main():
    assert canAttendMeetings([[0,30],[5,10],[15,20]]) == False
    assert canAttendMeetings([[7,10],[2,4]]) == True

if __name__ == "__main__":
    main()
