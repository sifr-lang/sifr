
# LeetCode 217: Contains Duplicate
# Python version

def containsDuplicate(nums: list[int]) -> bool:
    hashset = set()

    for n in nums:
        if n in hashset:
            return True
        hashset.add(n)
    return False



def main():
    assert containsDuplicate([1,2,3,1]) == True
    assert containsDuplicate([1,2,3,4]) == False

if __name__ == "__main__":
    main()
