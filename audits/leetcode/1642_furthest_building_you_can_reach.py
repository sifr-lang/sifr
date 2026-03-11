# LeetCode 1642: Furthest Building You Can Reach
# Python version

def furthestBuilding(heights: List[int], bricks: int, ladders: int) -> int:
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
    print("no test cases")

if __name__ == "__main__":
    main()
