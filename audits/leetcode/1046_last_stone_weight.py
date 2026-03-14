# LeetCode 1046: Last Stone Weight
# Python version

def lastStoneWeight(stones: List[int]) -> int:
    stones = [-s for s in stones]
    heapq.heapify(stones)

    while len(stones) > 1:
        first = heapq.heappop(stones)
        second = heapq.heappop(stones)
        if second > first:
            heapq.heappush(stones, first - second)

    stones.append(0)
    return abs(stones[0])

# There's a private _heapify_max method.
# https://github.com/python/cpython/blob/1170d5a292b46f754cd29c245a040f1602f70301/Lib/heapq.py#L198

def lastStoneWeight(stones):
    heapq._heapify_max(stones)
    while len(stones) > 1:
        max_stone = heapq._heappop_max(stones)
        diff = max_stone - stones[0]
        if diff:
            heapq._heapreplace_max(stones, diff)
        else:
            heapq._heappop_max(stones)
    
    stones.append(0)
    return stones[0]



def main():
    assert lastStoneWeight([2, 7, 4, 1, 8, 1]) == 1
    assert lastStoneWeight([1]) == 1

if __name__ == "__main__":
    main()
