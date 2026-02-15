# LeetCode 1929: Concatenation of Array
# Python version with test cases

def getConcatenation(nums: list[int]) -> list[int]:
    ans: list[int] = []
    for i in range(2):
        for j in range(len(nums)):
            ans.append(nums[j])
    return ans

def main():
    print(getConcatenation([1, 2, 1]))
    print(getConcatenation([1, 3, 2, 1]))

if __name__ == "__main__":
    main()
