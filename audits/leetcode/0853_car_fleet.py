# LeetCode 853: Car Fleet
# Python version

def carFleet(target: int, position: List[int], speed: List[int]) -> int:
    pair = [(p, s) for p, s in zip(position, speed)]
    pair.sort(reverse=True)
    stack = []
    for p, s in pair:  # Reverse Sorted Order
        stack.append((target - p) / s)
        if len(stack) >= 2 and stack[-1] <= stack[-2]:
            stack.pop()
    return len(stack)



def main():
    assert carFleet(12, [10,8,0,5,3], [2,4,1,1,3]) == 3

if __name__ == "__main__":
    main()
