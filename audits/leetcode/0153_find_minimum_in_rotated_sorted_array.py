
# LeetCode 153: Find Minimum In Rotated Sorted Array
# Python version

def findMin(nums: list[int]) -> int:
    start , end = 0, len(nums) - 1 
    curr_min = float("inf")
    
    while start  <  end :
        mid = start + (end - start ) // 2
        curr_min = min(curr_min,nums[mid])
        
        # right has the min 
        if nums[mid] > nums[end]:
            start = mid + 1
            
        # left has the  min 
        else:
            end = mid - 1 
            
    return min(curr_min,nums[start])



def main():
    assert findMin([3,4,5,1,2]) == 1
    assert findMin([4,5,6,7,0,1,2]) == 0

if __name__ == "__main__":
    main()
