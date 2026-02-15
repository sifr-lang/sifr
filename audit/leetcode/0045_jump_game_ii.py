# LeetCode 45: Jump Game II
# Python version with test cases

def jump(nums: list[int]) -> int:
    jumps: int = 0
    current_end: int = 0
    farthest: int = 0
    for i in range(len(nums) - 1):
        if i + nums[i] > farthest:
            farthest = i + nums[i]
        if i == current_end:
            jumps += 1
            current_end = farthest
    return jumps

def main():
    print(jump([2, 3, 1, 1, 4]))
    print(jump([2, 3, 0, 1, 4]))
    print(jump([1, 2, 3]))
    print(jump([0]))

if __name__ == "__main__":
    main()
