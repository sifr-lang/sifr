# LeetCode 1383: Maximum Performance Of A Team
# Python version

def maxPerformance(n: int, speed: List[int], efficiency: List[int], k: int) -> int:
    mod = 10 ** 9 + 7
    eng = []
    for eff, spd in zip(efficiency, speed):
        eng.append([eff, spd])
    eng.sort(reverse = True)
    
    res, speed = 0, 0
    minHeap = []
    
    for eff, spd in eng:
        if len(minHeap) == k:
            speed -= heapq.heappop(minHeap)
        speed += spd
        heapq.heappush(minHeap, spd)
        res = max(res, eff * speed)
    return res % mod



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
