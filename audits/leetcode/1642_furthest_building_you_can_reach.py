import heapq

# LeetCode 1642: Furthest Building You Can Reach
# Python version

def furthestBuilding(heights: list[int], bricks: int, ladders: int) -> int:
    heap = []

    for i in range(len(heights) - 1):
        diff = heights[i + 1] - heights[i]
        if diff <= 0:
            continue

        bricks -= diff
        heapq.heappush(heap, -diff)

        if bricks < 0:
            if ladders == 0:
                return i
            ladders -= 1
            bricks += -heapq.heappop(heap)
        
    return len(heights) - 1



def main():
    assert furthestBuilding([4, 2, 7, 6, 9, 14, 12], 5, 1) == 4
    assert furthestBuilding([4, 12, 2, 7, 3, 18, 20, 3, 19], 10, 2) == 7
    assert furthestBuilding([14, 3, 19, 3], 17, 0) == 3

if __name__ == "__main__":
    main()
