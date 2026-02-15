# LeetCode 128: Longest Consecutive Sequence
# Python version

def longestConsecutive(nums: List[int]) -> int:
    numSet = set(nums)
    longest = 0

    for n in numSet:
        # check if its the start of a sequence
        if (n - 1) not in numSet:
            length = 1
            while (n + length) in numSet:
                length += 1
            longest = max(length, longest)
    return longest



def main():
    print(longestConsecutive([100,4,200,1,3,2]))

if __name__ == "__main__":
    main()
