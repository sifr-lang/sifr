# LeetCode 179: Largest Number
# Python version

def largestNumber(nums: List[int]) -> str:
    for i, n in enumerate(nums):
        nums[i] = str(n)

    def compare(n1, n2):
        if n1 + n2 > n2 + n1:
            return -1
        else:
            return 1

    nums = sorted(nums, key = cmp_to_key(compare))

    return str(int("".join(nums)))



def main():
    assert largestNumber([10, 2],) == 210
    assert largestNumber([3, 30, 34, 5, 9],) == 9534330

if __name__ == "__main__":
    main()
