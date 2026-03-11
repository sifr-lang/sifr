# LeetCode 334: Increasing Triplet Subsequence
# Python version

def increasingTriplet(nums: List[int]) -> bool:
    first = float('inf')  # Initialize first to positive infinity
    second = float('inf')  # Initialize second to positive infinity
    
    for num in nums:
        if num <= first:
            first = num  # Update first if num is smaller or equal
        elif num <= second:
            second = num  # Update second if num is smaller or equal
        else:
            return True  # We found a triplet: first < second < num
    
    return False  # No triplet exists



def main():
    print(increasingTriplet([1,2,3,4,5]))
    print(increasingTriplet([5,4,3,2,1]))

if __name__ == "__main__":
    main()
