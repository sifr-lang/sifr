# LeetCode 1851: Minimum Interval To Include Each Query
# Python version

def minInterval(intervals: List[List[int]], queries: List[int]) -> List[int]:
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
    print("no test cases")

if __name__ == "__main__":
    main()
