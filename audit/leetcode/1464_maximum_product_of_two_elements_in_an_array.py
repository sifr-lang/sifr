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
    print(maxProduct([3,4,5,2]))
    print(maxProduct([1,5,4,5]))

if __name__ == "__main__":
    main()
