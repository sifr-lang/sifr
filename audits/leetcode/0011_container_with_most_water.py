# LeetCode 11: Container With Most Water
# Python version

def maxArea(height: List[int]) -> int:
    l, r = 0, len(height) - 1
    res = 0

    while l < r:
        res = max(res, min(height[l], height[r]) * (r - l))
        if height[l] < height[r]:
            l += 1
        elif height[r] <= height[l]:
            r -= 1
        
    return res



def main():
    assert maxArea([1,8,6,2,5,4,8,3,7]) == 49
    assert maxArea([1,1]) == 1

if __name__ == "__main__":
    main()
