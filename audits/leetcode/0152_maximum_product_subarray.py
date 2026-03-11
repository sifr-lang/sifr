# LeetCode 152: Maximum Product Subarray
# Python version with test cases

def maxProduct(nums: list[int]) -> int:
    result: int = nums[0]
    cur_max: int = nums[0]
    cur_min: int = nums[0]
    for i in range(1, len(nums)):
        val: int = nums[i]
        # When multiplied by negative, max becomes min and vice versa
        if val < 0:
            temp: int = cur_max
            cur_max = cur_min
            cur_min = temp
        if val > cur_max * val:
            cur_max = val
        else:
            cur_max = cur_max * val
        if val < cur_min * val:
            cur_min = val
        else:
            cur_min = cur_min * val
        if cur_max > result:
            result = cur_max
    return result

def main():
    print(maxProduct([2, 3, -2, 4]))
    print(maxProduct([-2, 0, -1]))
    print(maxProduct([-2, 3, -4]))
    print(maxProduct([2, -5, -2, -4, 3]))

if __name__ == "__main__":
    main()
