
# LeetCode 219: Contains Duplicate Ii
# Python version

def containsNearbyDuplicate(nums: list[int], k: int) -> bool:
    window = set()
    L = 0

    for R in range(len(nums)):
        if R - L > k:
            window.remove(nums[L])
            L += 1
        if nums[R] in window:
            return True
        window.add(nums[R])
    return False



def main():
    assert containsNearbyDuplicate([1,2,3,1], 3) == True
    assert containsNearbyDuplicate([1,2,3,1,2,3], 2) == False

if __name__ == "__main__":
    main()
