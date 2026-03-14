# LeetCode 456: 132 Pattern
# Python version

def find132pattern(nums: List[int]) -> bool:
    stack = [] # pair [num, curLeftMin], mono-decreasing stack
    curMin = nums[0]

    for n in nums:
        while stack and n >= stack[-1][0]:
            stack.pop()
        if stack and n < stack[-1][0] and n > stack[-1][1]:
            return True

        stack.append([n, curMin]) 
        curMin = min(n, curMin)

    return False



def main():
    assert find132pattern([1,2,3,4]) == False
    assert find132pattern([3,1,4,2]) == True

if __name__ == "__main__":
    main()
