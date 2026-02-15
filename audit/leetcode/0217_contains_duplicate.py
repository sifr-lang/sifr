# LeetCode 217: Contains Duplicate
# Python version

def containsDuplicate(nums: List[int]) -> bool:
    hashset = set()

    for n in nums:
        if n in hashset:
            return True
        hashset.add(n)
    return False



def main():
    print(containsDuplicate([1,2,3,1]))
    print(containsDuplicate([1,2,3,4]))

if __name__ == "__main__":
    main()
