# LeetCode 1464: Maximum Product Of Two Elements In An Array
# Python version

def maxProduct(nums: List[int]) -> int:
    high = secondHigh = 0
    for n in nums:
        if n > high:
            secondHigh = high
            high = n
        else:
            secondHigh = max(n, secondHigh)
    return (high - 1) * (secondHigh - 1)



def main():
    assert maxProduct([3,4,5,2]) == 12
    assert maxProduct([1,5,4,5]) == 16

if __name__ == "__main__":
    main()
