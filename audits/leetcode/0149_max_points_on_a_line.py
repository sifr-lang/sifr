# LeetCode 149: Max Points On A Line
# Python version

def maxPoints(points: List[List[int]]) -> int:
    # 1. For each pt determine if it lies on the longest line
    # 2. Count all pts with same slope
    # 3. Update result with max

    res = 1
    for i in range(len(points)):
        p1 = points[i]
        count = collections.defaultdict(int)
        for j in range(i + 1, len(points)):
            p2 = points[j]
            if p2[0] == p1[0]:
                slope = float("inf")
            else:
                slope = (p2[1] - p1[1]) / (p2[0] - p1[0])
            count[slope] += 1
            res = max(res, count[slope] + 1)
    return res



def main():
    assert maxPoints([[1, 1], [2, 2], [3, 3]]) == 3
    assert maxPoints([[1, 1], [3, 2], [5, 3], [4, 1], [2, 3], [1, 4]]) == 4

if __name__ == "__main__":
    main()
