
# LeetCode 134: Gas Station
# Python version with test cases

def canCompleteCircuit(gas: list[int], cost: list[int]) -> int:
    total_tank: int = 0
    current_tank: int = 0
    start: int = 0
    for i in range(len(gas)):
        diff: int = gas[i] - cost[i]
        total_tank += diff
        current_tank += diff
        if current_tank < 0:
            start = i + 1
            current_tank = 0
    if total_tank >= 0:
        return start
    return -1

def main():
    assert canCompleteCircuit([1, 2, 3, 4, 5], [3, 4, 5, 1, 2]) == 3
    assert canCompleteCircuit([2, 3, 4], [3, 4, 3]) == -1
    assert canCompleteCircuit([5, 1, 2, 3, 4], [4, 4, 1, 5, 1]) == 4

if __name__ == "__main__":
    main()
