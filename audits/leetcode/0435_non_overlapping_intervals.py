
# LeetCode 435: Non Overlapping Intervals
# Python version

def eraseOverlapIntervals(intervals: list[list[int]]) -> int:
    intervals.sort()
    res = 0
    prevEnd = intervals[0][1]
    for start, end in intervals[1:]:
        if start >= prevEnd:
            prevEnd = end
        else:
            res += 1
            prevEnd = min(end, prevEnd)
    return res



def main():
    assert eraseOverlapIntervals([[1,2],[2,3],[3,4],[1,3]]) == 1
    assert eraseOverlapIntervals([[1,2],[1,2],[1,2]]) == 2

if __name__ == "__main__":
    main()
