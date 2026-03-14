# LeetCode 55: Jump Game
# Python version with test cases

def canJump(nums: list[int]) -> bool:
    max_reach: int = 0
    for i in range(len(nums)):
        if i > max_reach:
            return False
        if i + nums[i] > max_reach:
            max_reach = i + nums[i]
    return True

def main():
    assert canJump([2, 3, 1, 1, 4]) == True
    assert canJump([3, 2, 1, 0, 4]) == False
    assert canJump([0]) == True
    assert canJump([2, 0, 0]) == True

if __name__ == "__main__":
    main()
