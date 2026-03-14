
# LeetCode 746: Min Cost Climbing Stairs
# Python version

def minCostClimbingStairs(cost: list[int]) -> int:
    for i in range(len(cost) - 3, -1, -1):
        cost[i] += min(cost[i + 1], cost[i + 2])

    return min(cost[0], cost[1])



def main():
    assert minCostClimbingStairs([10,15,20]) == 15

if __name__ == "__main__":
    main()
