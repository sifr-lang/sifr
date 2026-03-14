import heapq

# LeetCode 1851: Minimum Interval To Include Each Query
# Python version

def minInterval(intervals: list[list[int]], queries: list[int]) -> list[int]:
    intervals.sort()
    minHeap = []
    res = {}
    i = 0
    for q in sorted(queries):
        while i < len(intervals) and intervals[i][0] <= q:
            l, r = intervals[i]
            heapq.heappush(minHeap, (r - l + 1, r))
            i += 1

        while minHeap and minHeap[0][1] < q:
            heapq.heappop(minHeap)
        res[q] = minHeap[0][0] if minHeap else -1
    return [res[q] for q in queries]



def main():
    assert minInterval([[1, 4], [2, 4], [3, 6], [4, 4]], [2, 3, 4, 5]) == [3, 3, 1, 4]
    assert minInterval([[2, 3], [2, 5], [1, 8], [20, 25]], [2, 19, 5, 22]) == [2, -1, 4, 6]

if __name__ == "__main__":
    main()
