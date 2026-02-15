# LeetCode 973: K Closest Points To Origin
# Python version

def kClosest(points: List[List[int]], k: int) -> List[List[int]]:
    minHeap = []
    for x, y in points:
        dist = (x ** 2) + (y ** 2)
        minHeap.append((dist, x, y))
    
    heapq.heapify(minHeap)
    res = []
    for _ in range(k):
        _, x, y = heapq.heappop(minHeap)
        res.append((x, y))
    return res


def main():
    print("no test cases")

if __name__ == "__main__":
    main()
